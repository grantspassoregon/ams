use crate::prelude::Convert;
use address::prelude::load_bin;
use aid::prelude::Clean;
use galileo::Color;
use galileo::galileo_types::cartesian::{CartesianPoint2d, CartesianPoint3d, Point2d, Rect};
use galileo::galileo_types::geometry::CartesianGeometry2d;
use galileo::galileo_types::geometry::Geom;
use galileo::galileo_types::impls::{Contour, Polygon};
use galileo::layer::feature_layer::{Feature, symbol};
use galileo::render::render_bundle::RenderPrimitive;
use geo::algorithm::bounding_rect::BoundingRect;
use num_traits::AsPrimitive;
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

    pub fn from_shp_z<P: AsRef<Path>>(path: P, name: &str) -> Clean<Self> {
        let polygons = shapefile::read_shapes_as::<_, shapefile::PolygonZ>(path)?;
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

    pub fn load<P: AsRef<Path>>(path: P) -> Clean<Self> {
        let records = load_bin(path)?;
        let decode: Self = bincode::deserialize(&records[..])?;
        Ok(decode)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Clean<()> {
        address::prelude::save(self, path)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BoundaryView {
    pub geometry: galileo::galileo_types::impls::MultiPolygon<Point2d>,
    pub bounds: Rect,
    pub selected: bool,
}

impl BoundaryView {
    pub fn from_shp(shp: &Boundary) -> Option<Self> {
        let rect = shp.geometry.bounding_rect();
        let geometry = Convert::new(shp.geometry.clone()).geo_to_multipolygon();
        if let Some(bounds) = rect {
            let bounds = Convert::new(bounds).rect();
            Some(Self {
                geometry,
                bounds,
                selected: false,
            })
        } else {
            None
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Clean<Self> {
        let records = load_bin(path)?;
        let decode: Self = bincode::deserialize(&records[..])?;
        Ok(decode)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Clean<()> {
        address::prelude::save(self, path)
    }
}

impl galileo::galileo_types::geometry::Geometry for BoundaryView {
    type Point = Point2d;

    fn project<P: galileo::galileo_types::geo::Projection<InPoint = Self::Point> + ?Sized>(
        &self,
        projection: &P,
    ) -> Option<galileo::galileo_types::geometry::Geom<P::OutPoint>> {
        self.geometry.project(projection)
    }
}

impl CartesianGeometry2d<Point2d> for BoundaryView {
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

pub struct BoundarySymbol {}

impl BoundarySymbol {
    pub fn polygon(&self, feature: &BoundaryView) -> symbol::SimplePolygonSymbol {
        let selected = feature.selected;
        let stroke = {
            if selected {
                Color::BLUE
            } else {
                Color::from_hex("#ffcc00")
            }
        };
        let fill = Color::TRANSPARENT;
        symbol::SimplePolygonSymbol::new(fill)
                .with_stroke_color(stroke)
                .with_stroke_width(2.0)
                .with_stroke_offset(-1.0)
    }
}

impl symbol::Symbol<BoundaryView> for BoundarySymbol {
    fn render<'a, N, P>(
        &self,
        feature: &BoundaryView,
        geometry: &'a Geom<P>,
        min_resolution: f64,
    ) -> Vec<RenderPrimitive<'a, N, P, Contour<P>, Polygon<P>>>
    where
        N: AsPrimitive<f32>,
        P: CartesianPoint3d<Num = N> + Clone,
    {
        self.polygon(feature)
            .render(&(), geometry, min_resolution)
    }
}

impl Feature for BoundaryView {
    type Geom = Self;

    fn geometry(&self) -> &Self::Geom {
        self
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
pub struct CityLimitsView {
    pub geometry: galileo::galileo_types::impls::MultiPolygon<Point2d>,
    pub bounds: Rect,
}

impl CityLimitsView {
    pub fn from_shp(shp: &CityLimits) -> Option<Self> {
        let rect = shp.geometry.bounding_rect();
        let geometry = Convert::new(shp.geometry.clone()).geo_to_multipolygon();
        if let Some(bounds) = rect {
            let bounds = Convert::new(bounds).rect();
            Some(Self {
                geometry,
                bounds,
            })
        } else {
            None
        }
    }
}

impl galileo::galileo_types::geometry::Geometry for CityLimitsView {
    type Point = Point2d;

    fn project<P: galileo::galileo_types::geo::Projection<InPoint = Self::Point> + ?Sized>(
        &self,
        projection: &P,
    ) -> Option<galileo::galileo_types::geometry::Geom<P::OutPoint>> {
        self.geometry.project(projection)
    }
}

impl CartesianGeometry2d<Point2d> for CityLimitsView {
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

impl Feature for CityLimitsView {
    type Geom = Self;

    fn geometry(&self) -> &Self::Geom {
        self
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
