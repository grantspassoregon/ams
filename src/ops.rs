use crate::prelude::{AddressSource, TableView};
use address::prelude::{MatchRecord, MatchRecords, MatchStatus};
use strum::IntoEnumIterator;
use tracing::info;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Operations {
    pub compare: Compare,
    pub drift: bool,
    pub duplicates: bool,
    pub load: bool,
}

impl Operations {
    pub fn compare(ui: &mut egui::Ui) {
        if ui.button("Select Subject Addresses").clicked() {};
        ui.button("Select Comparison Addresses");
        ui.button("Run");
    }

    pub fn compare_visible(&self) -> bool {
        self.compare.visible
    }

    pub fn load_visible(&self) -> bool {
        self.load
    }

    pub fn toggle_compare(&mut self) {
        self.compare.toggle();
    }

    pub fn toggle_load(&mut self) {
        self.load = !self.load;
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
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
