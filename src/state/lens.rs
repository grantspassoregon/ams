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
    pub enter: Option<()>,
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
            enter: None,
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
            act::EguiAct::Be => tracing::trace!("Taking no action."),
        }
    }

    /// Receiver for an ['Act'] sent from the main event loop.
    pub fn enter(&mut self) {
        tracing::trace!("State for Enter set.");
        self.enter = Some(());
    }

    pub fn ams(&mut self, ui: &mut egui::Ui) {
        // let text_style = egui::TextStyle::Body;

        // The first time this functions runs, we want to record the focus points for later use,
        // but we do not want this logic to run every frame.
        // Create a new tree, but only swap it in for the actual tree if the flags field is empty.
        let mut tree = focus::Tree::new();
        // side panel id
        let (side_panel, side_panel_index) = tree.window();
        let parent_node = tree.node();
        tree.with_window(parent_node, side_panel);

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
            self.focus_tree.select(load.id);
            self.enter();
        }

        let sample = ui.button("Sample Data");
        tree.with_new_leaf(parent_node, &sample);
        self.focus_tree.focusable(&sample);

        // Register click as enter press.
        if sample.clicked() {
            tracing::info!("Sample clicked.");
            self.focus_tree.select(sample.id);
            self.enter();
        }

        // Create compare button.
        let compare = ui.button("Compare");
        tree.with_new_leaf(parent_node, &compare);
        self.focus_tree.focusable(&compare);

        if compare.clicked() {
            tracing::info!("Compare clicked.");
            self.focus_tree.select(compare.id);
            self.enter();
        }

        let drift = ui.button("Drift");
        tree.with_new_leaf(parent_node, &drift);
        self.focus_tree.focusable(&drift);

        if drift.clicked() {
            tracing::info!("Drift clicked.");
            self.focus_tree.select(drift.id);
            self.enter();
        }

        let duplicates = ui.button("Duplicates");
        tree.with_new_leaf(parent_node, &duplicates);
        self.focus_tree.focusable(&duplicates);

        if duplicates.clicked() {
            tracing::info!("Duplicates clicked.");
            self.focus_tree.select(duplicates.id);
            self.enter();
        }

        let lexis = ui.button("LexisNexis");
        tree.with_new_leaf(parent_node, &lexis);
        self.focus_tree.focusable(&lexis);

        if lexis.clicked() {
            tracing::info!("LexisNexis clicked.");
            self.focus_tree.select(lexis.id);
            self.enter();
        }

        let load_widget_id = egui::Id::new("load_widget");
        let load_widget = egui::Window::new("Load Data").id(load_widget_id);
        let mut load_tree = focus::Tree::new();
        let (load_widget_node, _) = load_tree.with_new_window();

        if self.operations.load_visible() {
            load_widget.show(ui.ctx(), |ui| {
                ui.heading("Address Data");
                let load_inner = ui.button("Load");
                let _ = load_tree.with_new_leaf(load_widget_node, &load_inner);
                load_tree.focusable(&load_inner);

                if load_inner.clicked() {
                    tracing::info!("LexisNexis clicked.");
                    self.focus_tree.select(load_inner.id);
                    self.enter();
                }

                if self.data.addresses.len() > 0 {
                    for (i, address) in self.data.addresses.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", self.data.address_sources[i]));
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
                if self.focus_tree.window_index() == 1 {
                    if let Some(_) = self.enter.take() {
                        tracing::info!("Enter detected in load widget.");
                        tracing::info!("Window index: {}", self.focus_tree.window_index());
                        if let Some(id) = self.focus_tree.current_leaf_id() {
                            tracing::info!("Current focus: {:?}", id);
                            tracing::info!("Inner leaf id: {:?}", load_inner.id);
                            if id == load_inner.id {
                                tracing::info!("Inner load button in focus.");
                                self.data.read_addresses();
                                self.notify.success("Selected addresses loaded!");
                                tracing::info!("Toast sent.");
                            }
                        }
                    }
                }
                self.focus_tree.update(&mut tree, load_tree);
            });
        }

        if self.operations.compare_visible() {
            egui::Window::new("Compare").show(ui.ctx(), |ui| {
                if ui.button("Run").clicked() {
                    let table = Some(self.data.compare(&self.operations.compare));
                    self.operations.compare.table = table;
                }
                self.operations.compare.combo(ui);
            });
        }

        if self.operations.lexis_visible() {
            egui::Window::new("LexisNexis").show(ui.ctx(), |ui| {
                self.operations.lexis.combo(ui);
            });
        }

        if self.operations.duplicates_visible() {
            egui::Window::new("Duplicates").show(ui.ctx(), |ui| {
                if ui.button("Run").clicked() {
                    tracing::info!("Run duplicates clicked.");
                }
                self.operations.duplicates(ui);
            });
        }

        egui::Window::new("Commands").show(ui.ctx(), |ui| self.command_view.show(ui));

        // Wire up enter to take action.
        if self.focus_tree.window_index() == 0 {
            if let Some(_) = self.enter.take() {
                tracing::info!("Enter detected in side panel.");
                tracing::info!("Side panel index: {side_panel_index}");
                tracing::info!("Window index: {}", self.focus_tree.window_index());
                if let Some(id) = self.focus_tree.select {
                    tracing::info!("Current focus: {:?}", id);
                    if id == load.id {
                        tracing::info!("Load button in focus.");
                        self.operations.toggle_load();
                    }
                    if id == sample.id {
                        tracing::info!("Sample button in focus.");
                        self.data.sample_data().unwrap();
                        self.notify.success("Sample data loaded!");
                        tracing::info!("Toast sent.");
                    }
                    if id == compare.id {
                        tracing::info!("Compare button in focus.");
                        self.operations.toggle_compare();
                    }
                    if id == drift.id {
                        tracing::info!("Drift button in focus.");
                        self.operations.toggle_drift();
                    }
                    if id == duplicates.id {
                        tracing::info!("Duplicates button in focus.");
                        self.operations.toggle_duplicates();
                    }
                    if id == lexis.id {
                        tracing::info!("LexisNexis button in focus.");
                        if self.operations.lexis.addresses.len() != self.data.addresses.len() {
                            self.operations.lexis.addresses = self.data.addresses.clone();
                            self.operations.lexis.sources = self.data.address_sources.clone();
                        }
                        self.operations.toggle_lexis();
                    }
                } else {
                    tracing::info!("Tree select is empty.");
                }
            }
        }

        if self.focus_tree.flags.is_empty() {
            self.focus_tree = tree;
            tracing::info!("Focus tree set.");
            tracing::info!("{:#?}", self.focus_tree);
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
