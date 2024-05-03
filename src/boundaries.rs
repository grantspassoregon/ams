use crate::prelude::Convert;
use aid::prelude::Clean;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Boundary {
    pub name: String,
    pub geometry: geo::geometry::MultiPolygon,
}

impl Boundary {
    pub fn new(name: &str, geometry: geo::geometry::MultiPolygon) -> Self {
        Self {
            name: name.to_owned(),
            geometry,
        }
    }

    pub fn from_shp<P: AsRef<Path>>(path: P, name: &str) -> Clean<Self> {
        let polygons = shapefile::read_shapes_as::<_, shapefile::Polygon>(path)?;
        let mut polys = Vec::new();
        for poly in polygons {
            let conv = Convert::new(poly);
            let geo_poly = conv.geo_polygons();
            polys.extend(geo_poly);
        }
        Ok(Self {
            name: name.to_owned(),
            geometry: polys.into(),
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Clean<()> {
        address::prelude::save(self, path)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CityLimits {
    pub geometry: geo::geometry::MultiPolygon,
}

impl CityLimits {
    pub fn from_shp<P: AsRef<Path>>(path: P) -> Clean<Self> {
        let polygons = shapefile::read_shapes_as::<_, shapefile::Polygon>(path)?;
        let mut polys = Vec::new();
        for poly in polygons {
            let conv = Convert::new(poly);
            let geo_poly = conv.geo_polygons();
            polys.extend(geo_poly);
        }
        Ok(Self {
            geometry: polys.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublicSafetyAgreement {
    pub geometry: geo::geometry::MultiPolygon,
}

impl PublicSafetyAgreement {
    pub fn from_shp<P: AsRef<Path>>(path: P) -> Clean<Self> {
        let polygons = shapefile::read_shapes_as::<_, shapefile::Polygon>(path)?;
        let mut polys = Vec::new();
        for poly in polygons {
            let conv = Convert::new(poly);
            let geo_poly = conv.geo_polygons();
            polys.extend(geo_poly);
        }
        Ok(Self {
            geometry: polys.into(),
        })
    }
}
