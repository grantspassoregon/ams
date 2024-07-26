use crate::controls::focus;
use crate::data;
use crate::prelude::{
    AddressPoints, AddressSource, Boundary, BoundaryView, Columnar, Filtration, TableView, Tabular,
};
use address::prelude::{
    Addresses, LexisNexis, LexisNexisItem, MatchRecord, MatchRecords, MatchStatus, Portable,
    SpatialAddresses,
};
use aid::prelude::*;
use geo::algorithm::contains::Contains;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, fmt};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::info;

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Operations {
    pub compare: Compare,
    /// The `drift` field indicates the drift widget is visible.
    pub drift: bool,
    /// The `duplicates` field indicates the duplicates widget is visible.
    pub duplicates: bool,
    /// The `load` field indicates the load widget is visible.
    pub load: bool,
    /// Contains the LexisNexis widget.
    pub lexis: Lexis,
    /// The [`AddressSource`] associated with the subject data.
    pub subject: AddressSource,
    /// The index of the addresses in the `addresses` field of [`Data`].
    pub subject_idx: usize,
    // pub table: Option<TableView<AddressPoints, AddressPoint, String>>,
}

impl Operations {
    pub fn duplicates(&mut self, ui: &mut egui::Ui, data: &mut data::Data) {
        ui.push_id("subject", |ui| {
            let run = ui.button("Run");
            if run.clicked() {
                tracing::info!("Run duplicates clicked.");
                let duplicates = AddressPoints::from(&SpatialAddresses::from(
                    &data.addresses[self.subject_idx].filter("duplicates")[..],
                ));
                let table = TableView::new(duplicates);
                table.view();
            }
            egui::ComboBox::from_label("Select subject source")
                .selected_text(format!("{}", self.subject.to_string()))
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
        // if let Some(t) = &mut self.table {
        //     t.table(ui);
        // }
    }

    pub fn compare_visible(&self) -> bool {
        self.compare.visible
    }

    pub fn load_visible(&self) -> bool {
        self.load
    }

    pub fn lexis_visible(&self) -> bool {
        self.lexis.visible
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
        self.lexis.visible = !self.lexis.visible;
    }

    pub fn toggle_drift(&mut self) {
        self.drift = !self.drift;
    }

    pub fn toggle_duplicates(&mut self) {
        self.duplicates = !self.duplicates;
    }

    pub fn load_widget(
        &mut self,
        ui: &mut egui::Ui,
        parent_tree: &mut focus::Tree,
        data: &mut data::Data,
        notify: &mut egui_notify::Toasts,
    ) {
        let mut tree = focus::Tree::new();
        let parent_node = tree.with_new_window();

        if self.load_visible() {
            egui::Window::new("Load Data")
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.heading("Address Data");
                    let load = ui.button("Load");
                    tree.with_new_leaf(parent_node, &load);
                    tree.focusable(&load);

                    if load.clicked() {
                        tracing::info!("Load Inner clicked.");
                        data.read_addresses();
                        notify.success("Selected addresses loaded!");
                        tracing::info!("Toast sent.");
                    }

                    if data.addresses.len() > 0 {
                        for (i, address) in data.addresses.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}", data.address_sources[i]));
                                ui.label(egui::RichText::new("■").color(egui::Color32::GREEN));
                                ui.label(format!("{} records", address.len()));
                            });
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("■").color(egui::Color32::RED));
                            ui.label("No data loaded.");
                        });
                    }
                    if parent_tree.enter.is_some() {
                        tracing::info!("Enter detected in load widget.");
                        if let Some(id) = parent_tree.current_leaf() {
                            tracing::info!("Current focus: {:?}", id);
                            tracing::info!("Inner leaf id: {:?}", load.id);
                            if id == load.id {
                                tracing::info!("Inner load button in focus.");
                                data.read_addresses();
                                notify.success("Selected addresses loaded!");
                                tracing::info!("Toast sent.");
                                parent_tree.enter = None;
                            }
                        }
                    }
                    if parent_tree.contains_new(&tree) {
                        parent_tree.graft(tree);
                        tracing::info!("Compare tree added.");
                    }
                });
        }
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
    pub fn combo(
        &mut self,
        ui: &mut egui::Ui,
        parent_tree: &mut focus::Tree,
        data: &mut data::Data,
    ) {
        let mut tree = focus::Tree::new();
        let parent_node = tree.with_new_window();
        ui.horizontal(|ui| {
            let run = ui.button("Run");
            tree.with_new_leaf(parent_node, &run);
            // Register button with focus tree.
            tree.focusable(&run);
            if run.clicked() {
                self.run(data);
            }

            let save = ui.button("Save");
            tree.with_new_leaf(parent_node, &save);
            // Register button with focus tree.
            tree.focusable(&save);
            if save.clicked() {
                self.save();
            }
            if parent_tree.enter.is_some() {
                tracing::info!("Enter detected in compare widget.");
                if let Some(id) = parent_tree.current_leaf() {
                    tracing::info!("Current focus: {:?}", id);
                    tracing::info!("Compare id: {:?}", run.id);
                    if id == run.id {
                        tracing::info!("Run compare button in focus.");
                        self.run(data);
                        // Clear the `enter` field after taking action.
                        parent_tree.enter = None;
                    }
                    if id == save.id {
                        tracing::info!("Save compare button in focus.");
                        self.save();
                        // Clear the `enter` field after taking action.
                        parent_tree.enter = None;
                    }
                }
            }
        });
        ui.push_id("subject", |ui| {
            egui::ComboBox::from_label("Select subject source")
                .selected_text(self.subject.to_string())
                .show_ui(ui, |ui| {
                    for (i, source) in AddressSource::iter().enumerate() {
                        if ui
                            .selectable_value(&mut self.subject, source.clone(), source.to_string())
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
                .selected_text(self.target.to_string())
                .show_ui(ui, |ui| {
                    for (i, target) in AddressSource::iter().enumerate() {
                        if ui
                            .selectable_value(&mut self.target, target.clone(), target.to_string())
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
        if parent_tree.contains_new(&tree) {
            parent_tree.graft(tree);
            tracing::info!("Compare tree added.");
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

    pub fn run(&mut self, data: &mut data::Data) {
        let table = Some(data.compare(&self));
        self.table = table;
    }

    /// Saves the comparison table to a csv file on the local machine.
    pub fn save(&self) {
        // Get path to current working directory.
        let path = env::current_dir().expect("Could not read current directory.");
        // Use the `rfd` crate to manage the file dialog.
        let file = rfd::FileDialog::new()
            // Restrict visible files to type "csv".
            .add_filter("csv", &["csv"])
            // Start the dialog view in the current working directory.
            .set_directory(&path)
            // Start with the default save name as "address_comparison.csv".
            .set_file_name("address_comparison.csv")
            .save_file();
        // From the file handle defined by the dialog...
        if let Some(path) = file {
            if let Some(mut view) = self.table.clone() {
                info!("Saving address comparison table.");
                // The `view` field in a `TableView` holds a view of the table data with
                // filters applied.
                view.view.to_csv(path).unwrap();
            }
        }
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
    visible: bool,
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
                visible: false,
            })
        } else {
            Err(Bandage::Hint(
                "Could not load lexis boundary view.".to_string(),
            ))
        }
    }

    pub fn combo(&mut self, ui: &mut egui::Ui, parent_tree: &mut focus::Tree, data: &data::Data) {
        if self.addresses.len() != data.addresses.len() {
            self.addresses = data.addresses.clone();
            self.sources = data.address_sources.clone();
        }
        let mut tree = focus::Tree::new();
        let parent_node = tree.with_new_window();
        if self.sources.is_empty() {
            ui.label("No address data loaded.");
        } else {
            egui::ComboBox::from_label("Select address source")
                .selected_text(format!("{}", self.sources[self.selected]))
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
                let run = ui.button("Run");
                tree.with_new_leaf(parent_node, &run);
                // Register button with focus tree.
                tree.focusable(&run);
                if run.clicked() {
                    self.run();
                }
                let save = ui.button("Save");
                tree.with_new_leaf(parent_node, &save);
                // Register button with focus tree.
                tree.focusable(&save);
                if save.clicked() {
                    self.save();
                }
                if parent_tree.enter.is_some() {
                    tracing::info!("Enter detected in lexis widget.");
                    if let Some(id) = parent_tree.current_leaf() {
                        tracing::info!("Current focus: {:?}", id);
                        tracing::info!("Run lexis id: {:?}", run.id);
                        if id == run.id {
                            tracing::info!("Run lexis button in focus.");
                            self.run();
                            // Clear the `enter` field after taking action.
                            parent_tree.enter = None;
                        }
                        if id == save.id {
                            tracing::info!("Save lexis button in focus.");
                            self.save();
                            // Clear the `enter` field after taking action.
                            parent_tree.enter = None;
                        }
                    }
                }
            });
        }
        if let Some(view) = &mut self.view {
            view.table(ui);
        }
        if parent_tree.contains_new(&tree) {
            parent_tree.graft(tree);
            tracing::info!("LexisNexis tree added.");
        }
    }

    /// Functionality for the run button in the Lexis Nexis widget.
    pub fn run(&mut self) {
        tracing::info!("Running LexisNexis.");
        // `records` and `other` will hold addresses within and without the LexisNexis boundary.
        // `records` are addresses inside City of Grants Pass service area.
        let mut records = Vec::new();
        // `other` are addresses outside the City of Grants Pass service area.
        let mut other = Vec::new();
        // `target` are the selected addresses to analzye.
        let target = &self.addresses[self.selected];
        // Convert to AddressPoints and then geo::geometry::Point type to access the spatial
        // operation `contains` in the `geo` crate.
        let ap = AddressPoints::from(target);
        let gp = ap
            .par_iter()
            .map(|v| v.geo_point())
            .collect::<Vec<geo::geometry::Point>>();
        // Use contains to determine whether each point is within the Lexis Nexis boundary.
        for (i, pt) in gp.iter().enumerate() {
            // info!("Point: {:#?}", pt);
            // info!("Contained: {}", self.boundary.geometry.contains(pt));
            if self.boundary.geometry.contains(pt) {
                // Push to `records` if within boundary.
                records.push(target[i].clone());
            } else {
                // Push to `other` if outside boundary.
                other.push(target[i].clone());
            }
        }

        // Convert back to `SpatialAddresses` to access the `lexisnexis` method for calculating the
        // Lexis Nexis table.
        let records = SpatialAddresses::from(&records[..]);
        tracing::info!("Inclusion records: {}", records.len());
        let other = SpatialAddresses::from(&other[..]);
        tracing::info!("Exclusion records: {}", other.len());
        // Package the address point results for delivery to the map window.
        self.address_pkg = Some(vec![records.clone(), other.clone()]);
        // Build the Lexis Nexis table.
        let lexis = records.lexis_nexis(&other).unwrap();
        tracing::info!("LexisNexis records: {}", lexis.len());
        // Load the Lexis Nexis table into a table view for display.
        let view = Some(TableView::new(lexis));
        // Copy the table view to the `view` field.
        self.view = view.clone();
        // Package the table view.
        self.package = view;
    }

    /// Saves the Lexis Nexis table to a csv file on the local machine.
    pub fn save(&self) {
        // Get path to current working directory.
        let path = env::current_dir().expect("Could not read current directory.");
        // Use the `rfd` crate to manage the file dialog.
        let file = rfd::FileDialog::new()
            // Restrict visible files to type "csv".
            .add_filter("csv", &["csv"])
            // Start the dialog view in the current working directory.
            .set_directory(&path)
            // Start with the default save name as "lexisnexis.csv".
            .set_file_name("lexisnexis.csv")
            .save_file();
        // From the file handle defined by the dialog...
        if let Some(path) = file {
            if let Some(mut view) = self.view.clone() {
                info!("Saving Lexis Nexis table.");
                // The `data` field in a `TableView` holds the complete table data, without
                // filters.
                view.data
                    // Write the LexisNexis table to a csv file.
                    .to_csv(path)
                    .expect("Could not save LexisNexis table to csv.");
            }
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
        LexisNexisColumns::iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
    }

    fn rows(&self) -> Vec<LexisNexisItem> {
        self.to_vec()
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

    fn id(&self) -> uuid::Uuid {
        self.id
    }
}

impl Filtration<LexisNexis, String> for LexisNexis {
    fn filter(&mut self, filter: &String) -> Self {
        info!("Filtering not implemented, ignoring {}", filter);
        self.clone()
    }
}
