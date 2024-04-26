use crate::prelude::{
    toggle_select, Columnar, Compare, Filtration, Parcels, TableConfig, TableView, Tabular,
};
use address::prelude::{
    Address, AddressStatus, GrantsPassSpatialAddresses, JosephineCountySpatialAddresses,
    MatchRecord, MatchRecords, MatchStatus, Portable, SpatialAddress, SpatialAddresses,
};
use egui::{Align, Layout, Sense, Slider, Ui};
use egui_extras::{Column, TableBuilder};
use galileo::layer::feature_layer::symbol::Symbol;
use galileo::render::point_paint::PointPaint;
use galileo::render::render_bundle::RenderPrimitive;
use galileo::Color;
use galileo_types::cartesian::CartesianPoint3d;
use galileo_types::geometry::Geom;
use galileo_types::impls::{Contour, Polygon};
use num_traits::AsPrimitive;
use rfd::FileDialog;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::info;

#[derive(Debug, Default, Clone)]
pub struct Data {
    pub addresses: Vec<SpatialAddresses>,
    pub address_sources: Vec<AddressSource>,
    pub compare: Option<TableView<MatchRecords, MatchRecord, String>>,
    pub parcels: Option<Arc<Parcels>>,
    pub selection: HashSet<usize>,
    pub target: AddressSource,
}

impl Data {
    pub fn read_addresses(&mut self) {
        let files = FileDialog::new()
            .add_filter("csv", &["csv"])
            .set_directory("/")
            .pick_file();

        let mut records = SpatialAddresses::default();
        if let Some(path) = files {
            if let Ok(values) = GrantsPassSpatialAddresses::from_csv(path.clone()) {
                if values.records.len() > records.records.len() {
                    self.address_sources.push(AddressSource::GrantsPass);
                    records = SpatialAddresses::from(&values.records[..]);
                }
            }
            if let Ok(values) = JosephineCountySpatialAddresses::from_csv(path.clone()) {
                if values.records.len() > records.records.len() {
                    self.address_sources.push(AddressSource::JosephineCounty);
                    records = SpatialAddresses::from(&values.records[..]);
                }
            }
            if records.records.len() > 0 {
                info!("Records found: {}", records.records.len());
                self.addresses.push(records);
            } else {
                info!("No records found.");
            }
        }
    }

    pub fn combo(&mut self, ui: &mut Ui, label: &str) {
        egui::ComboBox::from_label(label)
            .selected_text(format!("{:?}", self.target))
            .show_ui(ui, |ui| {
                for source in AddressSource::iter() {
                    ui.selectable_value(&mut self.target, source.clone(), format!("{source}"));
                }
            });
    }

    pub fn toggle_select(&mut self, row: usize, response: &egui::Response) {
        toggle_select(&mut self.selection, row, response);
    }

    pub fn compare(&mut self, data: &Compare) -> TableView<MatchRecords, MatchRecord, String> {
        let subject = &self.addresses[data.subject_idx].records[..];
        let target = &self.addresses[data.target_idx].records[..];
        let config = TableConfig::new().with_search().with_slider();
        let table = TableView::with_config(MatchRecords::compare(subject, target), config);
        self.compare = Some(table.clone());
        table
    }

    pub fn filter(&mut self, filter: &str) {
        if let Some(table) = &mut self.compare {
            table.data = table.data.clone().filter(filter);
        }
    }
}

impl Tabular<AddressSource> for Data {
    fn headers() -> Vec<String> {
        AddressSource::names()
    }

    fn rows(&self) -> Vec<AddressSource> {
        self.address_sources.clone()
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq, Hash, EnumIter)]
pub enum AddressSource {
    GrantsPass,
    JosephineCounty,
}

impl Default for AddressSource {
    fn default() -> Self {
        AddressSource::GrantsPass
    }
}

impl fmt::Display for AddressSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GrantsPass => write!(f, "City of Grants Pass"),
            Self::JosephineCounty => write!(f, "Josephine County"),
        }
    }
}

impl Columnar for AddressSource {
    fn names() -> Vec<String> {
        vec!["Address Source".to_owned()]
    }

    fn values(&self) -> Vec<String> {
        vec![format!("{self}")]
    }
}

#[derive(EnumIter, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MatchColumns {
    MatchStatus,
    Address,
    Subaddress,
    Floor,
    Building,
    Status,
    Latitude,
    Longitude,
}

impl MatchColumns {
    pub fn value(&self, record: &MatchRecord) -> String {
        match self {
            Self::MatchStatus => format!("{:?}", record.match_status),
            Self::Address => format!("{}", record.address_label),
            Self::Subaddress => format!("{:?}", record.subaddress_type),
            Self::Floor => format!("{:?}", record.floor),
            Self::Building => format!("{:?}", record.building),
            Self::Status => format!("{:?}", record.status),
            Self::Latitude => format!("{}", record.latitude),
            Self::Longitude => format!("{}", record.longitude),
        }
    }
}

impl fmt::Display for MatchColumns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MatchStatus => write!(f, "Match Status"),
            Self::Address => write!(f, "Address"),
            Self::Subaddress => write!(f, "Subaddress Type"),
            Self::Floor => write!(f, "Floor"),
            Self::Building => write!(f, "Building"),
            Self::Status => write!(f, "Status"),
            Self::Latitude => write!(f, "Latitude"),
            Self::Longitude => write!(f, "Longitude"),
        }
    }
}

impl Columnar for MatchRecord {
    fn names() -> Vec<String> {
        MatchColumns::iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<String>>()
    }

    fn values(&self) -> Vec<String> {
        MatchColumns::iter()
            .map(|v| v.value(self))
            .collect::<Vec<String>>()
    }
}

impl Tabular<MatchRecord> for MatchRecords {
    fn headers() -> Vec<String> {
        MatchRecord::names()
    }

    fn rows(&self) -> Vec<MatchRecord> {
        self.records.clone()
    }
}

impl Filtration<MatchRecords, String> for MatchRecords {
    fn filter(self, filter: &String) -> Self {
        MatchRecords::filter(self, filter)
    }
}

// impl Symbol<AddressPoint> for AddressSymbol {
//     fn render<'a, N, P>(
//         &self,
//         feature: &AddressPoint,
//         geometry: &'a Geom<P>,
//         _min_resolution: f64,
//         ) -> Vec<RenderPrimitive<'a, N, P, Contour<P>, Polygon<P>>>
//         where
//             N: AsPrimitive<f32>,
//             P: CartesianPoint3d<Num = N> + Clone,
//         {
//             let size = 7.0 as f32;
//             let mut primitives = Vec::new();
//             let Geom::Point(point) = geometry else {
//                 return primitives;
//             };
//             let color = match &feature.address.address.status() {
//                 AddressStatus::Current => Color::BLUE,
//                 AddressStatus::Other => Color::from_hex("#dbc200"),
//                 AddressStatus::Pending => Color::from_hex("#db00d4"),
//                 AddressStatus::Temporary => Color::from_hex("#db6e00"),
//                 AddressStatus::Retired => Color::from_hex("#ad0000"),
//                 AddressStatus::Virtual => Color::GREEN,
//                 };
//             primitives.push(RenderPrimitive::new_point_ref(
//                     point,
//                     PointPaint::circle(color, size),
//                     ));
//             primitives
//             }
// }
