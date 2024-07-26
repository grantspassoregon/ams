use crate::controls::act;
use crate::state::lens;
use egui_dock::{NodeIndex, SurfaceIndex};

pub type Tab = lens::Lens;

#[derive(Debug, Clone, Default)]
pub struct TabView;

#[derive(Debug, Clone, Default)]
pub enum ContextMenu {
    #[default]
    App,
    Map,
}

#[derive(Debug, Clone)]
pub struct TabContext {
    kind: ContextMenu,
    surface: SurfaceIndex,
    node: NodeIndex,
}

impl TabContext {
    pub fn new(kind: ContextMenu, surface: SurfaceIndex, node: NodeIndex) -> Self {
        Self {
            kind,
            surface,
            node,
        }
    }

    pub fn kind(&self) -> &ContextMenu {
        &self.kind
    }
}

#[derive(Debug)]
pub struct TabViewer<'a> {
    added_nodes: &'a mut Vec<TabContext>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    #[allow(unused_variables)]
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        "Operations".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ams(ui);
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, surface: SurfaceIndex, node: NodeIndex) {
        ui.set_min_width(120.0);
        ui.style_mut().visuals.button_frame = false;

        if ui.button("App").clicked() {
            self.added_nodes
                .push(TabContext::new(ContextMenu::App, surface, node));
        }

        if ui.button("Map").clicked() {
            self.added_nodes
                .push(TabContext::new(ContextMenu::Map, surface, node));
        }
    }

    // fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
    //     self.added_nodes.push((surface, node));
    // }
}

pub struct TabState {
    tree: egui_dock::DockState<Tab>,
    tab_index: usize,
    notify: egui_notify::Toasts,
}

impl TabState {
    pub fn new(lens: lens::Lens) -> Self {
        // Create a `DockState` with an initial tab "tab1" in the main `Surface`'s root node.
        let tree = egui_dock::DockState::new(vec![lens]);
        let tab_index = 0;
        let notify = egui_notify::Toasts::default();
        Self {
            tree,
            tab_index,
            notify,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut added_nodes = Vec::new();
        // Here we just display the `DockState` using a `DockArea`.
        // This is where egui handles rendering and all the integrations.
        //
        // We can specify a custom `Style` for the `DockArea`, or just inherit
        // all of it from egui.
        egui_dock::DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .show_add_popup(true)
            .style(egui_dock::Style::from_egui(ui.style().as_ref()))
            .show_inside(
                ui,
                &mut TabViewer {
                    added_nodes: &mut added_nodes,
                },
            );
        added_nodes.drain(..).for_each(|tab_context| {
            self.tree
                .set_focused_node_and_surface((tab_context.surface, tab_context.node));
            self.tree.push_to_focused_leaf(lens::Lens::new());
            self.tab_index += 1;
            self.notify.success("Tab added.");
        });
        self.notify.show(ui.ctx());
    }

    pub fn run_ui(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("Menu").show(ctx, |ui| {
            self.ui(ui);
        });
    }

    pub fn act(&mut self, act: &act::EguiAct) {
        if let Some((_, tab)) = self.tree.main_surface_mut().find_active() {
            tab.act(act);
        }
    }

    pub fn tab(&mut self) -> Option<&mut lens::Lens> {
        if let Some((_, tab)) = self.tree.find_active_focused() {
            Some(tab)
        } else {
            None
        }
    }
}

impl Default for TabState {
    fn default() -> Self {
        let lens = lens::Lens::new();
        let tree = egui_dock::DockState::new(vec![lens]);
        let tab_index = 0;
        let notify = egui_notify::Toasts::default();
        Self {
            tree,
            tab_index,
            notify,
        }
    }
}
