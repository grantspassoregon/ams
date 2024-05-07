use address::prelude::{
    Address, AddressStatus, MatchRecord, MatchRecords, MatchStatus, Point, SpatialAddress,
    SpatialAddresses,
};
use galileo::galileo_types::cartesian::{CartesianPoint2d, CartesianPoint3d, Point2d};
use galileo::galileo_types::geo::impls::GeoPoint2d;
use galileo::galileo_types::geo::{GeoPoint, NewGeoPoint, Projection};
use galileo::galileo_types::geometry::Geom;
use galileo::galileo_types::geometry_type::{
    AmbiguousSpace, GeoSpace2d, GeometryType, PointGeometryType,
};
use galileo::galileo_types::impls::{Contour, Polygon};
use galileo::layer::feature_layer::symbol::Symbol;
use galileo::layer::feature_layer::Feature;
use galileo::render::point_paint::PointPaint;
use galileo::render::render_bundle::RenderPrimitive;
use galileo::Color;
use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct AddressPoint {
    pub address: SpatialAddress,
    pub point: Point2d,
    pub geo_point: GeoPoint2d,
}

impl AddressPoint {
    pub fn geo_point(&self) -> geo::geometry::Point {
        let x = CartesianPoint2d::x(self);
        let y = CartesianPoint2d::y(self);
        geo::geometry::Point::new(x, y)
    }
}

impl From<&SpatialAddress> for AddressPoint {
    fn from(address: &SpatialAddress) -> Self {
        let point = Point2d::new(CartesianPoint2d::x(address), CartesianPoint2d::y(address));
        let geo_point = GeoPoint2d::latlon(
            galileo_types::geo::GeoPoint::lat(address),
            galileo_types::geo::GeoPoint::lon(address),
        );
        let address = address.clone();
        Self {
            address,
            point,
            geo_point,
        }
    }
}

impl GeoPoint for AddressPoint {
    type Num = f64;

    fn lat(&self) -> Self::Num {
        self.address.latitude
    }

    fn lon(&self) -> Self::Num {
        self.address.longitude
    }
}

impl CartesianPoint2d for AddressPoint {
    type Num = f64;

    fn x(&self) -> Self::Num {
        CartesianPoint2d::x(&self.address)
    }

    fn y(&self) -> Self::Num {
        CartesianPoint2d::y(&self.address)
    }
}

impl CartesianPoint3d for AddressPoint {
    type Num = f64;

    fn x(&self) -> Self::Num {
        CartesianPoint2d::x(self)
    }

    fn y(&self) -> Self::Num {
        CartesianPoint2d::y(self)
    }

    fn z(&self) -> Self::Num {
        match self.address.floor() {
            Some(x) => *x as f64 * 5.0,
            None => 5.0,
        }
    }
}

impl GeometryType for AddressPoint {
    type Type = PointGeometryType;
    type Space = AmbiguousSpace;
}

impl Feature for AddressPoint {
    type Geom = GeoPoint2d;

    fn geometry(&self) -> &Self::Geom {
        &self.geo_point
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AddressPoints {
    pub records: Vec<AddressPoint>,
}

impl From<&SpatialAddresses> for AddressPoints {
    fn from(addresses: &SpatialAddresses) -> Self {
        let records = addresses
            .records
            .iter()
            .map(AddressPoint::from)
            .collect::<Vec<AddressPoint>>();
        Self { records }
    }
}

pub struct AddressSymbol {}

impl Symbol<AddressPoint> for AddressSymbol {
    fn render<'a, N, P>(
        &self,
        feature: &AddressPoint,
        geometry: &'a Geom<P>,
        _min_resolution: f64,
    ) -> Vec<RenderPrimitive<'a, N, P, Contour<P>, Polygon<P>>>
    where
        N: AsPrimitive<f32>,
        P: CartesianPoint3d<Num = N> + Clone,
    {
        let size = 7.0 as f32;
        let mut primitives = Vec::new();
        let Geom::Point(point) = geometry else {
            return primitives;
        };
        let color = match &feature.address.status() {
            AddressStatus::Current => Color::BLUE,
            AddressStatus::Other => Color::from_hex("#dbc200"),
            AddressStatus::Pending => Color::from_hex("#db00d4"),
            AddressStatus::Temporary => Color::from_hex("#db6e00"),
            AddressStatus::Retired => Color::from_hex("#ad0000"),
            AddressStatus::Virtual => Color::from_hex("#32a852"),
        };
        primitives.push(RenderPrimitive::new_point_ref(
            point,
            PointPaint::circle(color, size),
        ));
        primitives
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchPoint {
    record: MatchRecord,
    geo_point: GeoPoint2d,
}

impl GeoPoint for MatchPoint {
    type Num = f64;

    fn lat(&self) -> Self::Num {
        galileo_types::geo::GeoPoint::lat(&self.record)
    }

    fn lon(&self) -> Self::Num {
        galileo_types::geo::GeoPoint::lon(&self.record)
    }
}

impl GeometryType for MatchPoint {
    type Type = PointGeometryType;
    type Space = GeoSpace2d;
}

impl Feature for MatchPoint {
    type Geom = GeoPoint2d;

    fn geometry(&self) -> &Self::Geom {
        &self.geo_point
    }
}

impl From<&MatchRecord> for MatchPoint {
    fn from(record: &MatchRecord) -> Self {
        let geo_point = GeoPoint2d::latlon(
            galileo_types::geo::GeoPoint::lat(record),
            galileo_types::geo::GeoPoint::lon(record),
        );
        let record = record.clone();
        Self { record, geo_point }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MatchPoints {
    pub records: Vec<MatchPoint>,
}

impl From<&MatchRecords> for MatchPoints {
    fn from(records: &MatchRecords) -> Self {
        let records = records
            .records
            .iter()
            .map(|r| r.into())
            .collect::<Vec<MatchPoint>>();
        Self { records }
    }
}

pub struct MatchSymbol {}

impl Symbol<MatchPoint> for MatchSymbol {
    fn render<'a, N, P>(
        &self,
        feature: &MatchPoint,
        geometry: &'a Geom<P>,
        _min_resolution: f64,
    ) -> Vec<RenderPrimitive<'a, N, P, Contour<P>, Polygon<P>>>
    where
        N: AsPrimitive<f32>,
        P: CartesianPoint3d<Num = N> + Clone,
    {
        let size = 7.0 as f32;
        let mut primitives = Vec::new();
        let Geom::Point(point) = geometry else {
            return primitives;
        };
        let color = match &feature.record.match_status {
            MatchStatus::Matching => Color::BLUE,
            MatchStatus::Divergent => Color::from_hex("#dbc200"),
            MatchStatus::Missing => Color::from_hex("#ad0000"),
        };
        primitives.push(RenderPrimitive::new_point_ref(
            point,
            PointPaint::circle(color, size),
        ));
        primitives
    }
}
