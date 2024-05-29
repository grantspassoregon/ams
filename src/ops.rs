use crate::prelude::{
    AddressPoints, AddressSource, Boundary, BoundaryView, Columnar, Filtration, TableView, Tabular,
};
use address::prelude::{
    Addresses, LexisNexis, LexisNexisItem, MatchRecord, MatchRecords, MatchStatus, Portable,
    SpatialAddress, SpatialAddresses,
};
use aid::prelude::*;
use geo::algorithm::contains::Contains;
use rayon::prelude::*;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::info;

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Operations {
    pub compare: Compare,
    pub drift: bool,
    pub duplicates: bool,
    pub load: bool,
    pub lexis: Lexis,
    pub lexis_on: bool,
}

impl Operations {
    pub fn compare_visible(&self) -> bool {
        self.compare.visible
    }

    pub fn load_visible(&self) -> bool {
        self.load
    }

    pub fn lexis_visible(&self) -> bool {
        self.lexis_on
    }

    pub fn drift_visible(&self) -> bool {
        self.drift
    }

    pub fn duplicates_visible(&self) -> bool {
        self.duplicates
    }

    pub fn toggle_compare(&mut self) {
        self.compare.toggle();
    }

    pub fn toggle_load(&mut self) {
        self.load = !self.load;
    }

    pub fn toggle_lexis(&mut self) {
        self.lexis_on = !self.lexis_on;
    }

    pub fn toggle_drift(&mut self) {
        self.drift = !self.drift;
    }

    pub fn toggle_duplicates(&mut self) {
        self.duplicates = !self.duplicates;
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Compare {
    pub subject: AddressSource,
    pub subject_idx: usize,
    pub target: AddressSource,
    pub target_idx: usize,
    pub table: Option<TableView<MatchRecords, MatchRecord, String>>,
    pub visible: bool,
    pub status: Option<MatchStatus>,
    pub status_pkg: Option<MatchStatus>,
    pub package: Option<TableView<MatchRecords, MatchRecord, String>>,
}

impl Compare {
    pub fn combo(&mut self, ui: &mut egui::Ui) {
        ui.push_id("subject", |ui| {
            egui::ComboBox::from_label("Select subject source")
                .selected_text(format!("{:?}", self.subject))
                .show_ui(ui, |ui| {
                    for (i, source) in AddressSource::iter().enumerate() {
                        if ui
                            .selectable_value(
                                &mut self.subject,
                                source.clone(),
                                format!("{source}"),
                            )
                            .clicked()
                        {
                            self.subject_idx = i;
                            info!("Subject set to {i}");
                        }
                    }
                });
        });
        ui.push_id("target", |ui| {
            egui::ComboBox::from_label("Select comparison source")
                .selected_text(format!("{:?}", self.target))
                .show_ui(ui, |ui| {
                    for (i, target) in AddressSource::iter().enumerate() {
                        if ui
                            .selectable_value(&mut self.target, target.clone(), format!("{target}"))
                            .clicked()
                        {
                            self.target_idx = i;
                            info!("Target set to {i}");
                        }
                    }
                });
        });
        self.filter_panel(ui);
        if let Some(t) = &mut self.table {
            t.table(ui);
        }
    }

    pub fn filter_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(t) = &mut self.table {
            ui.horizontal(|ui| {
                ui.label("Filter:");
                if ui
                    .radio_value(&mut t.filter, Some("matching".to_string()), "Matching")
                    .clicked()
                {
                    t.view = t.data.clone().filter("matching");
                    t.package = Some(t.view.clone());
                };
                if ui
                    .radio_value(&mut t.filter, Some("divergent".to_string()), "Divergent")
                    .clicked()
                {
                    t.view = t.data.clone().filter("divergent");
                    t.package = Some(t.view.clone());
                };
                if ui
                    .radio_value(&mut t.filter, Some("missing".to_string()), "Missing")
                    .clicked()
                {
                    t.view = t.data.clone().filter("missing");
                    t.package = Some(t.view.clone());
                };
                if ui.radio_value(&mut t.filter, None, "None").clicked() {
                    t.view = t.data.clone();
                    t.package = Some(t.view.clone());
                };
            });
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Lexis {
    pub boundary: Boundary,
    pub boundary_view: BoundaryView,
    pub addresses: Vec<SpatialAddresses>,
    pub sources: Vec<AddressSource>,
    pub selected: usize,
    pub view: Option<TableView<LexisNexis, LexisNexisItem, String>>,
    pub package: Option<TableView<LexisNexis, LexisNexisItem, String>>,
    pub address_pkg: Option<Vec<SpatialAddresses>>,
    pub boundary_pkg: Option<BoundaryView>,
}

impl Lexis {
    pub fn try_default() -> Clean<Self> {
        let boundary = Boundary::load("data/lexis_nexis_boundary.data")?;
        if let Some(boundary_view) = BoundaryView::from_shp(&boundary) {
            Ok(Self {
                boundary,
                boundary_view: boundary_view.clone(),
                addresses: Default::default(),
                sources: Default::default(),
                selected: Default::default(),
                view: None,
                package: None,
                address_pkg: None,
                boundary_pkg: Some(boundary_view),
            })
        } else {
            Err(Bandage::Hint(
                "Could not load lexis boundary view.".to_string(),
            ))
        }
    }

    pub fn combo(&mut self, ui: &mut egui::Ui) {
        if self.sources.is_empty() {
            ui.label("No address data loaded.");
        } else {
            egui::ComboBox::from_label("Select address source")
                .selected_text(format!("{:?}", self.sources[self.selected]))
                .show_ui(ui, |ui| {
                    for (i, source) in self.sources.iter().enumerate() {
                        if ui
                            .selectable_label(i == self.selected, format!("{source}"))
                            .clicked()
                        {
                            self.selected = i;
                            info!("Subject set to {i}");
                        }
                    }
                });
            ui.horizontal(|ui| {
                if ui.button("Run").clicked() {
                    let mut records = Vec::new();
                    let mut other = Vec::new();
                    let target = &self.addresses[self.selected];
                    let ap = AddressPoints::from(target);
                    let gp = ap
                        .records
                        .par_iter()
                        .map(|v| v.geo_point())
                        .collect::<Vec<geo::geometry::Point>>();
                    for (i, pt) in gp.iter().enumerate() {
                        // info!("Point: {:#?}", pt);
                        // info!("Contained: {}", self.boundary.geometry.contains(pt));
                        if self.boundary.geometry.contains(pt) {
                            records.push(target.records[i].clone());
                        } else {
                            other.push(target.records[i].clone());
                        }
                    }
                    let records = SpatialAddresses { records };
                    let other = SpatialAddresses { records: other };
                    self.address_pkg = Some(vec![records.clone(), other.clone()]);
                    let lexis = records.lexis_nexis(&other).unwrap();
                    let view = Some(TableView::new(lexis));
                    self.view = view.clone();
                    self.package = view;
                }
                if ui.button("Save").clicked() {
                    let path = std::env::current_dir().unwrap();
                    let file = FileDialog::new()
                        .add_filter("csv", &["csv"])
                        .set_directory(&path)
                        .set_file_name("lexisnexis.csv")
                        .save_file();
                    if let Some(path) = file {
                        if let Some(mut view) = self.view.clone() {
                            info!("Saving Lexis Nexis table.");
                            view.data.to_csv(path).unwrap();
                        }
                    }
                }
            });
        }
        if let Some(view) = &mut self.view {
            view.table(ui);
        }
    }
}

impl Default for Lexis {
    fn default() -> Self {
        Self::try_default().unwrap()
    }
}

impl Tabular<LexisNexisItem> for LexisNexis {
    fn headers() -> Vec<String> {
        LexisNexisItem::names()
    }

    fn rows(&self) -> Vec<LexisNexisItem> {
        self.records.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter, Deserialize, Serialize)]
pub enum LexisNexisColumns {
    NumberFrom,
    NumberTo,
    Directional,
    StreetName,
    StreetType,
    Community,
    Zip,
}

impl fmt::Display for LexisNexisColumns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Self::NumberFrom => "From Address Number",
            Self::NumberTo => "To Address Number",
            Self::Directional => "Directional Prefix",
            Self::StreetName => "Street Name",
            Self::StreetType => "Street Type",
            Self::Community => "Postal Community",
            Self::Zip => "Zip Code",
        };
        write!(f, "{}", msg)
    }
}

impl Columnar for LexisNexisItem {
    fn names() -> Vec<String> {
        LexisNexisColumns::iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
    }

    fn values(&self) -> Vec<String> {
        let number_from = format!("{}", self.address_number_from);
        let number_to = format!("{}", self.address_number_to);
        let mut directional = "".to_string();
        if let Some(dir) = &self.street_name_pre_directional {
            directional.push_str(dir);
        }
        let zip = format!("{}", self.zip_code);
        vec![
            number_from,
            number_to,
            directional,
            self.street_name.clone(),
            self.street_name_post_type.clone(),
            self.postal_community.clone(),
            zip,
        ]
    }
}

impl Filtration<LexisNexis, String> for LexisNexis {
    fn filter(self, filter: &String) -> Self {
        info!("Filtering not implemented, ignoring {}", filter);
        self
    }
}
