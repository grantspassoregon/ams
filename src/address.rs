use crate::table;
use address::prelude::{
    Address, AddressStatus, MatchRecord, MatchRecords, MatchStatus, SpatialAddress,
    SpatialAddresses,
};
use aid::error::Bandage;
use derive_more::{Deref, DerefMut};
use galileo::galileo_types::cartesian::{CartesianPoint2d, CartesianPoint3d, Point2d};
use galileo::galileo_types::geo::impls::GeoPoint2d;
use galileo::galileo_types::geo::{GeoPoint, NewGeoPoint};
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
use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

#[derive(
    Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, EnumIter, Serialize, Deserialize,
)]
pub enum AddressColumns {
    #[default]
    Label,
    Number,
    Directional,
    StreetName,
    StreetType,
    SubaddressType,
    SubaddressId,
    Zip,
    Status,
}

impl AddressColumns {
    pub fn names() -> Vec<String> {
        let mut values = Vec::new();
        for column in AddressColumns::iter() {
            values.push(format!("{column}"));
        }
        values
    }
}

impl fmt::Display for AddressColumns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Label => write!(f, "Label"),
            Self::Number => write!(f, "Number"),
            Self::Directional => write!(f, "Directional Prefix"),
            Self::StreetName => write!(f, "Street Name"),
            Self::StreetType => write!(f, "Street Type"),
            Self::SubaddressType => write!(f, "Subaddress Type"),
            Self::SubaddressId => write!(f, "Subaddress ID"),
            Self::Zip => write!(f, "Zip"),
            Self::Status => write!(f, "Status"),
        }
    }
}

// We convert the "column index" from the table view to this enum to take advantage of pattern
// matching over an index.  Commits a *faux pas* if the number does not match an index in the address columns.
impl TryFrom<usize> for AddressColumns {
    type Error = Bandage;
    fn try_from(index: usize) -> Result<Self, Self::Error> {
        // iterate through address columns
        let columns = Self::iter()
            // index the iterator
            .enumerate()
            // match the column indices
            .filter(|(i, _)| *i == index)
            // grab the corresponding address column enum
            .map(|(_, v)| v)
            // will only return at most one success
            .take(1)
            // but we have to collect it into a vector, cuz ?
            .collect::<Vec<AddressColumns>>();
        // Given index does not map to a valid column.
        if columns.is_empty() {
            // Return a *faux pas*.
            Err(Bandage::Hint("Empty columns.".to_string()))
        // Valid column found.
        } else {
            // Return success value.
            Ok(columns[0].clone())
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AddressPoint {
    pub address: SpatialAddress,
    pub id: uuid::Uuid,
    pub point: Point2d,
    pub geo_point: GeoPoint2d,
}

impl AddressPoint {
    pub fn geo_point(&self) -> geo::geometry::Point {
        let x = CartesianPoint2d::x(self);
        let y = CartesianPoint2d::y(self);
        geo::geometry::Point::new(x, y)
    }

    pub fn column<T: fmt::Display>(&self, columns: &AddressColumns) -> String {
        match *columns {
            AddressColumns::Label => format!("{}", self.address.label()),
            AddressColumns::Number => format!("{}", self.address.number()),
            AddressColumns::Directional => {
                if let Some(prefix) = self.address.directional() {
                    format!("{}", prefix)
                } else {
                    "".to_string()
                }
            }
            AddressColumns::StreetName => format!("{}", self.address.street_name()),
            AddressColumns::StreetType => {
                if let Some(value) = &self.address.street_type() {
                    format!("{}", value.abbreviate())
                } else {
                    "".to_string()
                }
            }
            AddressColumns::SubaddressType => {
                if let Some(subtype) = &self.address.subaddress_type() {
                    format!("{}", subtype)
                } else {
                    "".to_string()
                }
            }
            AddressColumns::SubaddressId => {
                if let Some(value) = &self.address.subaddress_id() {
                    format!("{}", value)
                } else {
                    "".to_string()
                }
            }
            AddressColumns::Zip => format!("{}", self.address.zip()),
            AddressColumns::Status => format!("{}", self.address.status()),
        }
    }

    pub fn columns(&self) -> Vec<String> {
        let mut values = Vec::new();
        for column in AddressColumns::iter() {
            values.push(self.column::<String>(&column));
        }
        values
    }
}

impl From<&SpatialAddress> for AddressPoint {
    fn from(address: &SpatialAddress) -> Self {
        let point = Point2d::new(CartesianPoint2d::x(address), CartesianPoint2d::y(address));
        let geo_point = GeoPoint2d::latlon(
            galileo_types::geo::GeoPoint::lat(address),
            galileo_types::geo::GeoPoint::lon(address),
        );
        let id = uuid::Uuid::new_v4();
        let address = address.clone();
        Self {
            address,
            id,
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

impl table::Columnar for AddressPoint {
    fn values(&self) -> Vec<String> {
        self.columns()
    }

    fn id(&self) -> uuid::Uuid {
        self.id
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize, Deref, DerefMut)]
pub struct AddressPoints(Vec<AddressPoint>);

impl AddressPoints {
    pub fn sort_by_col(&mut self, column_index: usize, reverse: bool) {
        // Parse the index to an address column.
        if let Ok(column) = AddressColumns::try_from(column_index) {
            // Match against the column type and sort.
            match column {
                AddressColumns::Label => {
                    if reverse {
                        self.sort_by(|a, b| b.address.label().cmp(&a.address.label()));
                    } else {
                        self.sort_by(|a, b| a.address.label().cmp(&b.address.label()));
                    }
                }
                AddressColumns::Number => {
                    if reverse {
                        self.sort_by(|a, b| b.address.number().cmp(&a.address.number()));
                    } else {
                        self.sort_by(|a, b| a.address.number().cmp(&b.address.number()));
                    }
                }
                AddressColumns::Directional => {
                    if reverse {
                        self.sort_by(|a, b| b.address.directional().cmp(&a.address.directional()));
                    } else {
                        self.sort_by(|a, b| a.address.directional().cmp(&b.address.directional()));
                    }
                }
                AddressColumns::StreetName => {
                    if reverse {
                        self.sort_by(|a, b| b.address.street_name().cmp(&a.address.street_name()));
                    } else {
                        self.sort_by(|a, b| a.address.street_name().cmp(&b.address.street_name()));
                    }
                }
                AddressColumns::StreetType => {
                    if reverse {
                        self.sort_by(|a, b| b.address.street_type().cmp(&a.address.street_type()));
                    } else {
                        self.sort_by(|a, b| a.address.street_type().cmp(&b.address.street_type()));
                    }
                }
                AddressColumns::SubaddressType => {
                    if reverse {
                        self.sort_by(|a, b| {
                            b.address
                                .subaddress_type()
                                .cmp(&a.address.subaddress_type())
                        });
                    } else {
                        self.sort_by(|a, b| {
                            a.address
                                .subaddress_type()
                                .cmp(&b.address.subaddress_type())
                        });
                    }
                }
                AddressColumns::SubaddressId => {
                    if reverse {
                        self.sort_by(|a, b| {
                            b.address.subaddress_id().cmp(&a.address.subaddress_id())
                        });
                    } else {
                        self.sort_by(|a, b| {
                            a.address.subaddress_id().cmp(&b.address.subaddress_id())
                        });
                    }
                }
                AddressColumns::Zip => {
                    if reverse {
                        self.sort_by(|a, b| b.address.zip().cmp(&a.address.zip()));
                    } else {
                        self.sort_by(|a, b| a.address.zip().cmp(&b.address.zip()));
                    }
                }
                AddressColumns::Status => {
                    if reverse {
                        self.sort_by(|a, b| b.address.status().cmp(&a.address.status()));
                    } else {
                        self.sort_by(|a, b| a.address.status().cmp(&b.address.status()));
                    }
                }
            }
        }
    }
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> aid::prelude::Clean<()> {
        tracing::info!("Serializing to binary.");
        address::prelude::save(self, path)
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> aid::prelude::Clean<Self> {
        tracing::info!("Deserializing from binary.");
        let vec: Vec<u8> = std::fs::read(path)?;
        let addresses: AddressPoints = bincode::deserialize(&vec[..])?;
        Ok(addresses)
    }
}

impl table::Tabular<AddressPoint> for AddressPoints {
    fn headers() -> Vec<String> {
        AddressColumns::names()
    }

    fn rows(&self) -> Vec<AddressPoint> {
        self.to_vec()
    }

    fn sort_by_col(&mut self, column_index: usize, reverse: bool) {
        self.sort_by_col(column_index, reverse);
    }
}

impl table::Filtration<AddressPoints, String> for AddressPoints {}

impl From<&SpatialAddresses> for AddressPoints {
    fn from(addresses: &SpatialAddresses) -> Self {
        let records = addresses
            .iter()
            .map(AddressPoint::from)
            .collect::<Vec<AddressPoint>>();
        Self(records)
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
