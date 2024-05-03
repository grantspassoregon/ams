use egui::{Align, Layout, Sense, Slider, Ui};
use egui_extras::{Column, TableBuilder};
use spreadsheet::prelude::{BeaData, BeaDatum};
use std::collections::HashSet;
use std::marker::PhantomData;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TableView<T: Tabular<U> + Filtration<T, V> + Clone, U: Columnar, V: Default> {
    pub data: T,
    pub view: T,
    pub search: String,
    pub selection: HashSet<usize>,
    pub target: usize,
    pub config: TableConfig,
    pub filter: Option<V>,
    pub package: Option<T>,
    phantom: PhantomData<U>,
}

impl<T: Tabular<U> + Default + Filtration<T, V> + Clone, U: Columnar + Default, V: Default>
    TableView<T, U, V>
{
    pub fn new(data: T) -> Self {
        let view = data.clone();
        let package = Some(data.clone());
        Self {
            data,
            view,
            package,
            ..Default::default()
        }
    }

    pub fn with_config(data: T, config: TableConfig) -> Self {
        let view = data.clone();
        let package = Some(data.clone());
        Self {
            data,
            view,
            config,
            package,
            ..Default::default()
        }
    }

    fn toggle_row_selection(&mut self, row_index: usize, row_response: &egui::Response) {
        if row_response.clicked() {
            if self.selection.contains(&row_index) {
                self.selection.remove(&row_index);
            } else {
                self.selection.insert(row_index);
            }
        }
    }

    pub fn search_panel(&mut self, ui: &mut Ui) {
        if self.config.search {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search"));
                if ui.button("X").clicked() {
                    self.search = Default::default();
                }
            });
        }
    }

    pub fn searchable(&mut self) -> &mut Self {
        self.config.search = true;
        self
    }

    pub fn slider(&mut self, ui: &mut Ui, num_rows: usize) -> bool {
        let mut track_item = false;
        if self.config.slider {
            if num_rows == 0 {
                ui.label("Tracker disabled.");
            } else {
                ui.horizontal(|ui| {
                    track_item |= ui
                        .add(Slider::new(&mut self.target, 0..=(num_rows - 1)))
                        .dragged();
                    if ui.button("|<").clicked() {
                        self.target = 0;
                        track_item = true;
                    };
                    if ui.button(">|").clicked() {
                        self.target = num_rows - 1;
                        track_item = true;
                    };
                });
            }
        }
        track_item
    }

    pub fn with_slider(&mut self) -> &mut Self {
        self.config.slider = true;
        self
    }

    pub fn table(&mut self, ui: &mut Ui) {
        let num_rows = self.view.len();
        let track_item = self.slider(ui, num_rows);
        let headers = T::headers();
        let mut rows = self.view.rows();
        if !self.search.is_empty() {
            rows = self.contains(&self.search);
        }
        self.search_panel(ui);
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .columns(Column::auto(), headers.len());
        if track_item {
            table = table.scroll_to_row(self.target, Some(Align::Center));
        }

        table
            .header(20.0, |mut header| {
                headers
                    .iter()
                    .map(|v| {
                        header.col(|ui| {
                            ui.strong(v);
                        })
                    })
                    .for_each(drop);
            })
            .body(|body| {
                body.rows(20., rows.len(), |mut row| {
                    let row_index = row.index();
                    row.set_selected(self.selection.contains(&row_index));
                    let columns = &rows[row_index].values();
                    columns
                        .iter()
                        .map(|v| {
                            row.col(|ui| {
                                ui.label(v);
                            });
                        })
                        .for_each(drop);
                    self.toggle_row_selection(row_index, &row.response());
                });
            });
    }

    pub fn contains(&self, fragment: &str) -> Vec<U> {
        let mut data = Vec::new();
        let rows = self.view.rows();
        for row in rows {
            let mut contains = false;
            let cols = row.values();
            for col in cols {
                let mut value = col;
                let mut frag = fragment.to_string();
                if !self.config.case_sensitive {
                    value = value.to_lowercase();
                    frag = frag.to_lowercase();
                }
                if value.contains(&frag) {
                    contains = true;
                }
            }
            if contains {
                data.push(row);
            }
        }
        data
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableConfig {
    pub search: bool,
    pub slider: bool,
    pub case_sensitive: bool,
}

impl TableConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_search(mut self) -> Self {
        self.search = true;
        self
    }

    pub fn with_slider(mut self) -> Self {
        self.slider = true;
        self
    }

    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }
}

pub trait Tabular<T: Columnar> {
    fn headers() -> Vec<String>;
    fn rows(&self) -> Vec<T>;
    fn len(&self) -> usize {
        self.rows().len()
    }
}

impl Tabular<BeaDatum> for BeaData {
    fn headers() -> Vec<String> {
        BeaDatum::names()
    }

    fn rows(&self) -> Vec<BeaDatum> {
        self.records()
    }
}

pub trait Columnar {
    fn names() -> Vec<String>;
    fn values(&self) -> Vec<String>;
}

impl Columnar for BeaDatum {
    fn names() -> Vec<String> {
        Self::names()
    }

    fn values(&self) -> Vec<String> {
        Self::columns(self)
    }
}

pub trait Filtration<T, U> {
    fn filter(self, filter: &U) -> T;
}
