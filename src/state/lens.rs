use crate::controls::{act, command, focus};
use crate::prelude::{AddressPoint, AddressPoints, Parcels, TableConfig, TableView};
use crate::{data, ops};
use aid::prelude::Clean;
// use derive_more::{Deref, DerefMut};
// use egui::{Context, Id, TextStyle};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Lens {
    pub addresses: Option<AddressPoints>,
    pub address_table: Option<TableView<AddressPoints, AddressPoint, String>>,
    pub counter: i32,
    /// Command view window.
    pub command_view: command::CommandView,
    pub focus_tree: focus::Tree,
    pub focus_counter: bool,
    pub focus_parcels: bool,
    // pub panel: Option<Panel<AddressPoint>>,
    pub parcels: Option<Arc<Parcels>>,
    pub operations: ops::Operations,
    pub data: data::Data,
    notify: egui_notify::Toasts,
}

impl Lens {
    pub fn new() -> Self {
        // let vec = include_bytes!("../data/addresses.data");
        // let addresses: Option<AddressPoints> = match bincode::deserialize(&vec[..]) {
        //     Ok(data) => Some(data),
        //     Err(e) => {
        //         tracing::info!("{:#?}", e.to_string());
        //         None
        //     }
        // };

        // let mut panel = None;
        let mut address_table = None;
        let addresses = match AddressPoints::load("data/addresses.data") {
            Ok(data) => {
                // panel = Some(Panel::new(data.records.clone()));
                let config = TableConfig::new()
                    .checked()
                    .resizable()
                    .with_search()
                    .striped()
                    .with_slider();
                address_table = Some(TableView::with_config(data.clone(), config));
                tracing::trace!("Records read: {}", data.len());
                Some(data)
            }
            Err(e) => {
                tracing::info!("Could not read records: {}", e.to_string());
                None
            }
        };

        let parcels = match Parcels::load("data/parcels.data") {
            Ok(data) => Some(Arc::new(data)),
            Err(_) => None,
        };

        let command_tree = command::CommandMode::new();
        let command_table = command::CommandTable::from(&command_tree);
        let command_view = command::CommandView::from(&command_table);

        Self {
            addresses,
            address_table,
            counter: Default::default(),
            command_view,
            focus_tree: focus::Tree::new(),
            focus_counter: true,
            focus_parcels: true,
            // panel,
            parcels,
            operations: Default::default(),
            data: Default::default(),
            notify: Default::default(),
        }
    }

    pub fn in_focus(&mut self, id: egui::Id) -> bool {
        self.focus_tree.in_focus(&id)
    }

    pub fn act(&mut self, act: &act::EguiAct) {
        match *act {
            act::EguiAct::Right => {
                let _ = self.focus_tree.next_node();
                self.focus_tree.select_current();
            }
            act::EguiAct::Left => {
                let _ = self.focus_tree.previous_node();
                self.focus_tree.select_current();
            }
            act::EguiAct::Up => self.focus_tree.select_previous(),
            act::EguiAct::Down => self.focus_tree.select_next(),
            act::EguiAct::Next => self.focus_tree.select_next_node(),
            act::EguiAct::Previous => self.focus_tree.select_previous_node(),
            act::EguiAct::NextWindow => self.focus_tree.select_next_window(),
            act::EguiAct::PreviousWindow => self.focus_tree.select_previous_window(),
            act::EguiAct::NextRow => {
                if let Some(table) = &mut self.address_table {
                    tracing::info!("Selecting next row.");
                    table.select_next();
                }
            }
            act::EguiAct::PreviousRow => {
                if let Some(table) = &mut self.address_table {
                    tracing::info!("Selecting previous row.");
                    table.select_previous();
                }
            }
            act::EguiAct::InspectTree => self.focus_tree.inspect(),
            act::EguiAct::Be => tracing::trace!("Taking no action."),
        }
    }

    /// Logic for the load widget.
    pub fn load_widget(&mut self, ui: &mut egui::Ui) {
        let mut tree = focus::Tree::new();
        let parent_node = tree.with_new_window();
        // Create load button.
        let load = ui.button("Load Data");
        tree.with_new_leaf(parent_node, &load);
        // On initial load, select this button.
        tree.select(load.id);

        // Register button with focus tree.
        self.focus_tree.focusable(&load);
        // Register click as enter press.
        if load.clicked() {
            tracing::info!("Load clicked.");
            self.operations.toggle_load();
        }
        // Load widget logic.
        self.operations
            .load_widget(ui, &mut self.focus_tree, &mut self.data, &mut self.notify);
    }

    /// Logic for the sample data widget.
    pub fn sample_widget(&mut self, ui: &mut egui::Ui) {
        let mut tree = focus::Tree::new();
        let parent_node = tree.with_new_window();
        let sample = ui.button("Sample Data");
        tree.with_new_leaf(parent_node, &sample);
        self.focus_tree.focusable(&sample);

        // Register click as enter press.
        if sample.clicked() {
            tracing::info!("Sample clicked.");
            self.data
                .sample_data()
                .expect("Could not open sample data.");
            self.notify.success("Sample data loaded!");
        }
        if self.focus_tree.contains_new(&tree) {
            self.focus_tree.graft(tree);
            tracing::info!("Sample widget tree added.");
        }
    }

    /// Logic for the compare widget.
    pub fn compare_widget(&mut self, ui: &mut egui::Ui) {
        let mut tree = focus::Tree::new();
        let parent_node = tree.with_new_window();
        // Create compare button.
        let compare = ui.button("Compare");
        tree.with_new_leaf(parent_node, &compare);
        self.focus_tree.focusable(&compare);

        if compare.clicked() {
            tracing::info!("Compare clicked.");
            self.operations.toggle_compare();
        }
        if self.operations.compare_visible() {
            egui::Window::new("Compare")
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    self.operations
                        .compare
                        .combo(ui, &mut self.focus_tree, &mut self.data);
                });
        }
        if self.focus_tree.contains_new(&tree) {
            self.focus_tree.graft(tree);
            tracing::info!("Sample widget tree added.");
        }
    }

    pub fn ams(&mut self, ui: &mut egui::Ui) {
        // let text_style = egui::TextStyle::Body;

        // The first time this functions runs, we want to record the focus points for later use,
        // but we do not want this logic to run every frame.
        // Create a new tree, but only swap it in for the actual tree if the flags field is empty.
        let mut tree = focus::Tree::new();
        // side panel id
        let parent_node = tree.with_new_window();

        // Load widgets.
        self.load_widget(ui);
        self.sample_widget(ui);
        self.compare_widget(ui);

        let drift = ui.button("Drift");
        tree.with_new_leaf(parent_node, &drift);
        self.focus_tree.focusable(&drift);

        if drift.clicked() {
            tracing::info!("Drift clicked.");
            self.operations.toggle_drift();
        }

        let duplicates = ui.button("Duplicates");
        tree.with_new_leaf(parent_node, &duplicates);
        self.focus_tree.focusable(&duplicates);

        if duplicates.clicked() {
            tracing::info!("Duplicates clicked.");
            self.operations.toggle_duplicates();
        }

        let lexis = ui.button("LexisNexis");
        tree.with_new_leaf(parent_node, &lexis);
        self.focus_tree.focusable(&lexis);

        if lexis.clicked() {
            tracing::info!("LexisNexis clicked.");
            self.operations.toggle_lexis();
        }

        if self.operations.lexis_visible() {
            egui::Window::new("LexisNexis")
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    self.operations
                        .lexis
                        .combo(ui, &mut self.focus_tree, &self.data);
                });
        }

        if self.operations.duplicates_visible() {
            egui::Window::new("Duplicates")
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    if ui.button("Run").clicked() {
                        tracing::info!("Run duplicates clicked.");
                    }
                    self.operations.duplicates(ui, &mut self.data);
                });
        }

        egui::Window::new("Commands")
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| self.command_view.show(ui));

        // Wire up enter to take action.
        // if let Some(_) = self.enter.take() {
        //     tracing::info!("Enter detected in side panel.");
        //     if let Some(id) = self.focus_tree.select {
        //         tracing::info!("Current focus: {:?}", id);
        //         if id == load.id {
        //             tracing::info!("Load button in focus.");
        //             self.operations.toggle_load();
        //         }
        //         if id == sample.id {
        //             tracing::info!("Sample button in focus.");
        //             self.data.sample_data().unwrap();
        //             self.notify.success("Sample data loaded!");
        //             tracing::info!("Toast sent.");
        //         }
        //         if id == compare.id {
        //             tracing::info!("Compare button in focus.");
        //             self.operations.toggle_compare();
        //         }
        //         if id == drift.id {
        //             tracing::info!("Drift button in focus.");
        //             self.operations.toggle_drift();
        //         }
        //         if id == duplicates.id {
        //             tracing::info!("Duplicates button in focus.");
        //             self.operations.toggle_duplicates();
        //         }
        //         if id == lexis.id {
        //             tracing::info!("LexisNexis button in focus.");
        //             self.operations.toggle_lexis();
        //         }
        //     } else {
        //         tracing::info!("Tree select is empty.");
        //     }
        // }

        if self.focus_tree.contains_new(&tree) {
            self.focus_tree = tree;
            tracing::info!("Focus tree updated.");
        }

        self.notify.show(ui.ctx());
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Clean<()> {
        address::utils::save(&self, path)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Clean<Self> {
        let records = address::utils::load_bin(path)?;
        let decode: Self = bincode::deserialize(&records[..])?;
        Ok(decode)
    }
}
