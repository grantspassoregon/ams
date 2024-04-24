use crate::prelude::*;
use galileo::layer::feature_layer::Feature;
use galileo_types::cartesian::{CartesianPoint2d, Point2d, Rect};
use galileo_types::geometry::CartesianGeometry2d;
use galileo_types::impls::{Contour, MultiPolygon};
use geo::geometry::Geometry;
use geojson::FeatureReader;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use num_traits::Num;
use polite::{FauxPas, Polite};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use shapefile::record::polygon::Polygon;
use std::fs::File;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Owner {
    #[serde(rename(deserialize = "NAME"))]
    pub name: Option<String>,
    #[serde(rename(deserialize = "MapNum"))]
    pub id: String,
}

impl TryFrom<shapefile::dbase::Record> for Owner {
    type Error = FauxPas;
    fn try_from(shp: shapefile::dbase::Record) -> Polite<Self> {
        let mut name = None;
        match shp.get("NAME") {
            Some(shapefile::dbase::FieldValue::Character(Some(owner))) => {
                name = Some(owner.to_owned());
            }
            Some(_) => {
                info!("Owner name missing.");
            }
            None => {
                info!("Unexpected None at name.");
            }
        };
        let mut map_id = None;
        match shp.get("MapNum") {
            Some(shapefile::dbase::FieldValue::Character(Some(mapnum))) => {
                map_id = Some(mapnum.to_owned());
            }
            Some(_) => {
                info!("Map id missing.");
            }
            None => {
                info!("Unexpected None at id.");
            }
        }
        if let Some(id) = map_id {
            Ok(Owner { name, id })
        } else {
            Err(FauxPas::Unknown)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Parcel {
    pub owner: Owner,
    pub geometry: MultiPolygon<Point2d>,
    pub bounds: Rect,
    pub selected: bool,
}

impl Parcel {
    // pub fn read_geo(parcel: &Polygon) -> Geometry {
    //     let geo_poly: geo::MultiPolygon<f64> = parcel.clone().into();
    //     let geo: Geometry = geo_poly.into();
    //     geo
    // }

    //from EPSG:2270
    // pub fn to_epsg3857(mut geo: Geometry, from: &str) -> Geometry {
    //     geo.transform_crs_to_crs(from, "EPSG:3857").unwrap();
    //     geo
    // }

    pub fn read_record(geo: Geometry, record: shapefile::dbase::Record) -> Polite<Self> {
        let owner = Owner::try_from(record)?;
        let mut multipoly = None;
        let mut boundary = None;
        match &geo {
            Geometry::MultiPolygon(polys) => {
                let (mp, rect) = Convert::new(polys.clone()).bounded_multipolygon();
                multipoly = Some(mp);
                boundary = Some(rect);
            }
            _ => {
                info!("Not implemented.");
            }
        }
        if let Some(geometry) = multipoly {
            if let Some(bounds) = boundary {
                let parcel = Parcel {
                    owner,
                    geometry,
                    bounds,
                    selected: false,
                };
                Ok(parcel)
            } else {
                Err(FauxPas::Unknown)
            }
        } else {
            Err(FauxPas::Unknown)
        }
    }
}

impl galileo_types::geometry::Geometry for Parcel {
    type Point = Point2d;

    fn project<P: galileo_types::geo::Projection<InPoint = Self::Point> + ?Sized>(
        &self,
        projection: &P,
    ) -> Option<galileo_types::geometry::Geom<P::OutPoint>> {
        self.geometry.project(projection)
    }
}

impl CartesianGeometry2d<Point2d> for Parcel {
    fn is_point_inside<Other: CartesianPoint2d<Num = f64>>(
        &self,
        point: &Other,
        tolerance: f64,
    ) -> bool {
        if !self.bounds.contains(point) {
            return false;
        }

        self.geometry.is_point_inside(point, tolerance)
    }

    fn bounding_rectangle(&self) -> Option<Rect> {
        Some(self.bounds)
    }
}

impl Feature for Parcel {
    type Geom = Self;

    fn geometry(&self) -> &Self::Geom {
        self
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Parcels {
    pub records: Vec<Parcel>,
}

impl Parcels {
    pub fn from_geojson<P: AsRef<Path>>(path: P) -> Polite<Self> {
        let file = File::open(path)?;
        let reader = FeatureReader::from_reader(file);

        let mut records = Vec::new();
        let mut dropped = 0;
        let spinner = ProgressBar::new_spinner();
        for parcel in reader.deserialize()? {
            match parcel {
                Ok(lot) => records.push(lot),
                Err(e) => {
                    info!("Record dropped: {}.", e.to_string());
                    dropped += 1;
                }
            }
            spinner.tick();
        }
        info!("Records dropped: {}.", dropped);

        Ok(Parcels { records })
    }

    // pub fn from_shp<P: AsRef<Path>>(path: P, transform: Option<&str>) -> Polite<Self> {
    //     let polygons = shapefile::read_as::<_, shapefile::Polygon, shapefile::dbase::Record>(path).unwrap();
    //     let records = polygons
    //         .par_iter()
    //         .progress()
    //         .map(|v| {
    //             let mut geo = Parcel::read_geo(&v.0);
    //             if let Some(crs) = transform {
    //                 geo = Parcel::to_epsg3857(geo, crs);
    //             }
    //             let record = v.1.clone();
    //             Parcel::read_record(geo, record).unwrap()
    //         })
    //         .collect::<Vec<Parcel>>();
    //     Ok(Self { records })
    // }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Polite<()> {
        info!("Serializing to binary.");
        let encode = bincode::serialize(self)?;
        info!("Writing to file.");
        std::fs::write(path, encode)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Polite<Self> {
        info!("Deserializing from binary.");
        let vec: Vec<u8> = std::fs::read(path)?;
        let parcels: Parcels = bincode::deserialize(&vec[..])?;
        Ok(parcels)
    }
}
