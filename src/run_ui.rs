use crate::prelude::{Data, Operations};
use address::prelude::Portable;
use egui::{Align, Color32, Context, Layout, RichText, ScrollArea, Sense, Slider, TextStyle, Ui};
use egui_extras::{Column, TableBuilder};
use itertools::sorted;
use std::collections::{BTreeMap, HashMap, HashSet};
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Debug, Default)]
pub struct UiState {
    pub counter: i32,
    pub data: Data,
    pub operations: Operations,
}

impl UiState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self, ui: &Context) {
        let text_style = TextStyle::Body;

        egui::Window::new("AMS").show(ui, |ui| {
            ui.heading("Operations");
            if ui.button("Load Data").clicked() {
                self.operations.toggle_load();
            }
            if ui.button("Sample Data").clicked() {
                self.data.sample_data().unwrap();
            }
            if ui.button("Compare").clicked() {
                self.operations.toggle_compare();
            };
            if ui.button("Drift").clicked() {
                self.operations.toggle_drift();
            };
            if ui.button("Duplicates").clicked() {
                self.operations.toggle_duplicates();
            };
            if ui.button("LexisNexis").clicked() {
                if self.operations.lexis.addresses.len() != self.data.addresses.len() {
                    self.operations.lexis.addresses = self.data.addresses.clone();
                    self.operations.lexis.sources = self.data.address_sources.clone();
                }
                self.operations.toggle_lexis();
            };
        });

        if self.operations.load_visible() {
            egui::Window::new("Load Data").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Address Data");
                    if ui.button("Load").clicked {
                        self.data.read_addresses();
                    };
                });
                if self.data.addresses.len() > 0 {
                    for (i, address) in self.data.addresses.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", self.data.address_sources[i]));
                            ui.label(RichText::new("■").color(Color32::GREEN));
                            ui.label(format!("{} records", address.records.len()));
                        });
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("■").color(Color32::RED));
                        ui.label("No data loaded.");
                    });
                }
            });
        }

        if self.operations.compare_visible() {
            egui::Window::new("Compare").show(ui, |ui| {
                if ui.button("Run").clicked() {
                    let table = Some(self.data.compare(&self.operations.compare));
                    self.operations.compare.table = table;
                }
                self.operations.compare.combo(ui);
            });
        }

        if self.operations.lexis_visible() {
            egui::Window::new("LexisNexis").show(ui, |ui| {
                self.operations.lexis.combo(ui);
            });
        }
    }
}

use std::fmt::Display;
use std::hash::Hash;
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct HashPanel<K, V>
where
    K: Clone + PartialEq + Eq + PartialOrd + Ord + Hash + Display + Default,
    V: Clone + PartialEq + Eq + PartialOrd + Ord + Hash + Display + Default,
{
    pub data: BTreeMap<K, V>,
    pub key: Option<K>,
    pub selected: HashSet<V>,
    pub search: String,
    pub target: usize,
    pub value: V,
}

// impl<K: Eq + std::hash::Hash + Ord + Clone + std::fmt::Display, V: std::fmt::Display + Clone + Default + Eq + std::hash::Hash> HashPanel<K, V> {
impl<K, V> HashPanel<K, V>
where
    K: Clone + PartialEq + Eq + PartialOrd + Ord + Hash + Display + Default,
    V: Clone + PartialEq + Eq + PartialOrd + Ord + Hash + Display + Default,
{
    pub fn new(data: BTreeMap<K, V>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    pub fn combo(&mut self, ui: &mut Ui, label: String) {
        egui::ComboBox::from_label(label)
            .selected_text(format!("{}", self.value))
            .show_ui(ui, |ui| {
                for (_, val) in &self.data {
                    ui.selectable_value(&mut self.value, val.clone(), format!("{}", val));
                }
            });
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let mut panel = self.clone();
        if !self.search.is_empty() {
            panel.contains(&self.search);
        }
        let keys: Vec<&K> = sorted(panel.data.keys().into_iter()).collect();
        let num_rows = keys.len();
        let mut track_item = false;
        let mut scroll_top = false;
        let mut scroll_bottom = false;
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search"));
            if ui.button("X").clicked() {
                self.search = Default::default();
            }
        });
        if num_rows == 0 {
            ui.label("Tracker disabled.");
        } else {
            ui.horizontal(|ui| {
                track_item |= ui
                    .add(Slider::new(&mut self.target, 0..=(num_rows - 1)))
                    .dragged();
                scroll_top |= ui.button("|<").clicked();
                scroll_bottom |= ui.button(">|").clicked();
            });
        }

        ui.separator();
        ScrollArea::vertical()
            .max_height(400.)
            .show(ui, |ui| {
                if scroll_top {
                    ui.scroll_to_cursor(Some(Align::TOP));
                }
                ui.vertical(|ui| {
                    if num_rows == 0 {
                        ui.label("No data to display.");
                    } else {
                        for item in 0..=(num_rows - 1) {
                            if track_item && item == self.target {
                                let response = ui.selectable_value(
                                    &mut self.value,
                                    self.data[keys[item]].clone(),
                                    format!("{}: {}", keys[item], self.data[keys[item]]),
                                );
                                response.scroll_to_me(Some(Align::Center));
                                self.value = self.data[keys[item]].clone();
                                self.key = Some(keys[item].clone());
                            } else {
                                ui.selectable_value(
                                    &mut self.value,
                                    self.data[keys[item]].clone(),
                                    format!("{}: {}", keys[item], self.data[keys[item]]),
                                );
                                // ui.label(format!("{}: {}", keys[item], self.data[keys[item]]));
                            }
                        }
                    }
                });

                if scroll_bottom {
                    ui.scroll_to_cursor(Some(Align::BOTTOM));
                }
            })
            .inner;

        ui.separator();
        ui.label(format!("Value selected: {}", self.value));
    }

    pub fn entry_contains(fragment: &str, entry: (&K, &mut V)) -> bool {
        let key_str = entry.0.to_string();
        let val_str = entry.1.to_string();
        if key_str.contains(fragment) | val_str.contains(fragment) {
            true
        } else {
            false
        }
    }

    pub fn contains(&mut self, fragment: &str) {
        self.data.retain(|k, v| {
            let key = k.to_string().to_lowercase();
            let val = v.to_string().to_lowercase();
            if key.contains(fragment) | val.contains(fragment) {
                true
            } else {
                false
            }
        });
    }

    pub fn table(&mut self, ui: &mut Ui) {
        let mut panel = self.clone();
        if !self.search.is_empty() {
            panel.contains(&self.search);
        }
        let num_rows = panel.data.len();
        let mut track_item = false;
        let mut scroll_top = false;
        let mut scroll_bottom = false;
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search"));
            if ui.button("X").clicked() {
                self.search = Default::default();
            }
        });
        if num_rows == 0 {
            ui.label("Tracker disabled.");
        } else {
            ui.horizontal(|ui| {
                track_item |= ui
                    .add(Slider::new(&mut self.target, 0..=(num_rows - 1)))
                    .dragged();
                scroll_top |= ui.button("|<").clicked();
                scroll_bottom |= ui.button(">|").clicked();
                if ui.button("Clear").clicked() {
                    self.selected = HashSet::new();
                }
            });
        }

        ui.separator();

        let data = panel.data.clone();
        let keys = data.keys().collect::<Vec<&K>>();
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto().at_least(100.))
            .column(Column::auto().at_least(100.));
        if track_item {
            table = table.scroll_to_row(self.target, Some(Align::Center));
        }
        if scroll_top {
            table = table.scroll_to_row(0, Some(Align::BOTTOM));
        }
        if scroll_bottom {
            table = table.scroll_to_row(self.data.len(), Some(Align::BOTTOM));
        }
        table.body(|body| {
            body.rows(20., panel.data.len(), |mut row| {
                let row_index = row.index();
                row.set_selected(self.selected.contains(&panel.data[keys[row_index]]));
                row.col(|ui| {
                    ui.label(format!("{}", keys[row_index]));
                });
                row.col(|ui| {
                    ui.label(format!("{}", panel.data[keys[row_index]]));
                });
                self.toggle_row_selection(panel.data[keys[row_index]].clone(), &row.response());
            });
        });
    }

    pub fn toggle_row_selection(&mut self, target: V, row_response: &egui::Response) {
        if row_response.clicked() {
            if self.selected.contains(&target) {
                self.selected.remove(&target);
            } else {
                self.selected.insert(target);
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Panel<T> {
    pub data: HashMap<Uuid, T>,
    pub selected: HashSet<Uuid>,
    pub search: String,
    pub target: usize,
    pub value: Option<T>,
}

impl<T: PartialOrd + PartialEq + Clone + std::fmt::Display + Card + Default> Panel<T> {
    pub fn new(data: Vec<T>) -> Self {
        let data = data
            .iter()
            .map(|v| {
                let k = Uuid::new_v4();
                (k, v.clone())
            })
            .collect::<HashMap<Uuid, T>>();
        Self {
            data,
            ..Default::default()
        }
    }

    pub fn combo(&mut self, ui: &mut Ui, label: String) {
        let mut values = self
            .data
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(Uuid, T)>>();
        values.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let mut selected = if let Some(value) = &self.value {
            value.clone()
        } else {
            values[0].1.clone()
        };
        egui::ComboBox::from_label(label)
            .selected_text(format!("{}", selected))
            .show_ui(ui, |ui| {
                for value in values {
                    ui.selectable_value(&mut selected, value.1.clone(), format!("{}", value.1));
                }
            });
        self.value = Some(selected);
    }

    pub fn table(&mut self, ui: &mut Ui) {
        let mut panel = self.clone();
        if !self.search.is_empty() {
            panel.contains(&self.search);
        }
        let num_rows = panel.data.len();
        let mut track_item = false;
        let mut scroll_top = false;
        let mut scroll_bottom = false;
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search"));
            if ui.button("X").clicked() {
                self.search = Default::default();
            }
        });
        if num_rows == 0 {
            ui.label("Tracker disabled.");
        } else {
            ui.horizontal(|ui| {
                track_item |= ui
                    .add(Slider::new(&mut self.target, 0..=num_rows))
                    .dragged();
                scroll_top |= ui.button("|<").clicked();
                scroll_bottom |= ui.button(">|").clicked();
                if ui.button("Clear").clicked() {
                    self.selected = HashSet::new();
                }
            });
        }

        ui.separator();

        let data = panel.data.clone();
        let mut values = data
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(Uuid, T)>>();
        values.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto().at_least(100.));
        if track_item {
            table = table.scroll_to_row(self.target, Some(Align::Center));
        }
        if scroll_top {
            table = table.scroll_to_row(0, Some(Align::BOTTOM));
        }
        if scroll_bottom {
            table = table.scroll_to_row(self.data.len(), Some(Align::BOTTOM));
        }
        table.body(|body| {
            body.rows(20., panel.data.len(), |mut row| {
                let row_index = row.index();
                row.set_selected(self.selected.contains(&values[row_index].0));
                row.col(|ui| {
                    // ui.label(format!("{}", panel.data[keys[row_index]]));
                    values[row_index].1.show(ui);
                });
                self.toggle_row_selection(&values[row_index].0, &row.response());
            });
        });
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let mut panel = self.clone();
        if !self.search.is_empty() {
            panel.contains(&self.search);
        }
        let num_rows = panel.data.len();
        let mut track_item = false;
        let mut scroll_top = false;
        let mut scroll_bottom = false;
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search"));
            if ui.button("X").clicked() {
                self.search = Default::default();
            }
        });
        if num_rows == 0 {
            ui.label("Tracker disabled.");
        } else {
            ui.horizontal(|ui| {
                track_item |= ui
                    .add(Slider::new(&mut self.target, 0..=(num_rows - 1)))
                    .dragged();
                scroll_top |= ui.button("|<").clicked();
                scroll_bottom |= ui.button(">|").clicked();
            });
        }

        ui.separator();
        let data = panel.data.clone();
        let keys = data.keys().collect::<Vec<&Uuid>>();
        ScrollArea::vertical()
            .max_height(400.)
            .show(ui, |ui| {
                if scroll_top {
                    ui.scroll_to_cursor(Some(Align::TOP));
                }
                ui.vertical(|ui| {
                    if num_rows == 0 {
                        ui.label("No data to display.");
                    } else {
                        for item in 0..=(num_rows - 1) {
                            if track_item && item == self.target {
                                let response = ui.selectable_value(
                                    &mut self.value,
                                    Some(panel.data[keys[item]].clone()),
                                    format!("{}", panel.data[keys[item]]),
                                );
                                response.scroll_to_me(Some(Align::Center));
                                self.value = Some(panel.data[keys[item]].clone());
                                self.toggle_row_selection(keys[item], &response);
                            } else {
                                ui.selectable_value(
                                    &mut self.value,
                                    Some(panel.data[keys[item]].clone()),
                                    format!("{}", panel.data[keys[item]]),
                                );
                                // ui.label(format!("{}: {}", keys[item], self.data[keys[item]]));
                            }
                        }
                    }
                });

                if scroll_bottom {
                    ui.scroll_to_cursor(Some(Align::BOTTOM));
                }
            })
            .inner;

        ui.separator();
        ui.label(if let Some(value) = &self.value {
            format!("Value selected: {}", value)
        } else {
            format!("No value selected.")
        });
    }

    pub fn contains(&mut self, fragment: &str) {
        self.data.retain(|k, v| {
            let key = k.to_string().to_lowercase();
            let val = v.to_string().to_lowercase();
            if key.contains(fragment) | val.contains(fragment) {
                true
            } else {
                false
            }
        });
    }

    pub fn toggle_row_selection(&mut self, target: &Uuid, row_response: &egui::Response) {
        if row_response.clicked() {
            if self.selected.contains(target) {
                self.selected.remove(target);
            } else {
                self.selected.insert(*target);
            }
        }
    }

    pub fn values(&self) -> Vec<String> {
        self.selected
            .iter()
            .map(|k| format!("{}", self.data[k]))
            .collect::<Vec<String>>()
    }
}

pub trait Card {
    fn show(&self, ui: &mut Ui);
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Copy)]
pub struct SearchConfig {
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Default, Hash)]
pub struct Year(i32);

impl std::fmt::Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&i32> for Year {
    fn from(value: &i32) -> Self {
        Self(*value)
    }
}

impl Year {
    pub fn years(values: &[i32]) -> Vec<Self> {
        values.iter().map(|v| Self::from(v)).collect::<Vec<Self>>()
    }

    pub fn to_strings(years: &HashSet<Year>) -> Vec<String> {
        years
            .iter()
            .map(|v| format!("{}", v.0))
            .collect::<Vec<String>>()
    }
}

impl Card for Year {
    fn show(&self, ui: &mut Ui) {
        ui.label(format!("{}", self));
    }
}
