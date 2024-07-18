use egui::Id;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

/// The `Tree` struct tracks focus points in the user interface, and facilitates navigation.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    /// The `flags` field indicates if a given window has been loaded into the tree.
    pub flags: HashMap<Uuid, bool>,
    /// The `leaves` field holds focus points of type ['Leaf'].
    pub leaves: HashMap<Uuid, Leaf>,
    /// The `nodes` field holds focus points of type ['Node'].
    pub nodes: HashMap<Uuid, Node>,
    /// The `windows` field holds the ids of all windows in the user interface.
    pub windows: Vec<Uuid>,
    /// The `select` field indicates a change in the current focus ['egui::Id`].
    pub select: Option<Id>,
    /// The `current_leaf` field holds the current leaf in focus [`egui::Id`].
    pub current_leaf: Option<Id>,
    // Tracks the currently selected node.
    node_index: usize,
    // Tracks the currently selected window.
    window_index: usize,
}

impl Tree {
    /// Creates a new `Tree` struct using the default impl.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn graft(&mut self, branch: Tree) {
        self.leaves.extend(branch.leaves);
        self.nodes.extend(branch.nodes);
        self.windows.extend(branch.windows);
    }

    /// Creates a [`Leaf`] from an `id` of type [`egui::Id`].
    pub fn leaf(&mut self, id: Id) -> Uuid {
        Leaf::from_id(id, self)
    }

    /// Registers a [`Node`] in the user interface.
    pub fn node(&mut self) -> Uuid {
        Node::with_tree(self)
    }

    /// Creates a new window in the Tree.
    pub fn window(&mut self) -> (Uuid, usize) {
        // Create unique id for window.
        let id = Uuid::new_v4();
        // Push id to vector of window ids in self.
        self.windows.push(id);
        // Insert a flag for the window indicating it is not yet loaded.
        self.flags.insert(id, false);
        // Maybe unnecessary to return.
        (id, self.windows.len() - 1)
    }

    pub fn window_index(&self) -> usize {
        self.window_index
    }

    /// Sets the active focus point to an `id` of type [`egui::Id`].
    pub fn select(&mut self, id: Id) {
        self.select = Some(id);
        self.current_leaf = self.select;
    }

    /// Returns the active focus point.
    pub fn selected(&self) -> Option<Id> {
        self.select
    }

    /// Clears the active focus point.
    pub fn clear_selected(&mut self) {
        self.select = None;
    }

    pub fn in_focus(&self, id: &Id) -> bool {
        if let Some(value) = self.selected() {
            value == *id
        } else {
            false
        }
    }

    pub fn focusable(&mut self, response: &egui::Response) {
        if self.in_focus(&response.id) {
            // Request focus on button.
            tracing::trace!("Requesting focus for {:#?}", response.id);
            response.request_focus();
            // Reset select field.
            tracing::trace!("Clearing select.");
            self.clear_selected();
        }
    }

    pub fn with_leaf(&mut self, node: Uuid, leaf: Uuid) {
        Node::with_leaf(node, leaf, self);
    }

    pub fn with_new_leaf(&mut self, node: Uuid, leaf: &egui::Response) -> Uuid {
        let leaf_id = self.leaf(leaf.id);
        self.with_leaf(node, leaf_id);
        leaf_id
    }

    pub fn with_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn with_window(&mut self, node: Uuid, window: Uuid) {
        let node = self.nodes.get_mut(&node);
        if let Some(n) = node {
            n.with_window(window);
        }
    }

    pub fn with_new_window(&mut self) -> (Uuid, usize) {
        let (window_id, window_index) = self.window();
        let node_id = self.node();
        self.with_window(node_id, window_id);
        (node_id, window_index)
    }

    pub fn get_window(&self, window: Uuid) -> Vec<Uuid> {
        self.nodes
            .iter()
            // .map(|(k, v)| (k, v))
            .filter(|(_, v)| v.window == Some(window))
            .map(|(k, _)| k.clone())
            .collect::<Vec<Uuid>>()
    }

    /// Returns the [`Uuid`] of the current window.
    pub fn current_window(&self) -> Uuid {
        self.windows[self.window_index]
    }

    pub fn try_current_window(&self) -> Option<Uuid> {
        if self.windows.is_empty() {
            tracing::info!("Current window is empty.");
            None
        } else if self.windows.len() < self.window_index + 1 {
            tracing::info!("Window index out of bounds.");
            None
        } else {
            Some(self.windows[self.window_index])
        }
    }

    pub fn current_window_index(&self) -> usize {}

    /// Advance focus to the next window in the `window_index`.  Wraps to the first window if at
    /// the end of the queue.
    pub fn next_window(&mut self) -> Uuid {
        if self.window_index + 1 > self.windows.len() - 1 {
            // Loop around to the beginning if at the end.
            self.window_index = 0;
        } else {
            self.window_index += 1;
        }
        self.windows[self.window_index]
    }

    /// Move focus to the previous window in the `window_index`.  Wraps to last window if at the
    /// beginning of the queue.
    pub fn previous_window(&mut self) -> Uuid {
        if self.window_index == 0 {
            self.window_index = self.windows.len() - 1;
        } else {
            self.window_index -= 1;
        }
        self.windows[self.window_index]
    }

    /// Returns the [`Uuid`] of the current node.
    pub fn current_node(&self) -> Uuid {
        let id = self.current_window();
        let nodes = self.get_window(id);
        nodes[self.node_index]
    }

    /// Advances focus to the [`Uuid`] of the next [`Node`] in the `node_index`.  Wraps to the
    /// first node if at the end of the queue.
    pub fn next_node(&mut self) -> Uuid {
        let id = self.current_window();
        let nodes = self.get_window(id);
        if self.node_index == (nodes.len() - 1) {
            self.node_index = 0;
        } else {
            self.node_index += 1;
        }
        nodes[self.node_index]
    }

    /// Move focus to the [`Uuid`] of the previous [`Node`] in the `node_index`.  Wraps to the last
    /// node if at the beginning of the queue.
    pub fn previous_node(&mut self) -> Uuid {
        let id = self.current_window();
        let nodes = self.get_window(id);
        if self.node_index == 0 {
            self.node_index = nodes.len() - 1;
        } else {
            self.node_index -= 1;
        }
        nodes[self.node_index]
    }

    /// Advances focus to the next child node of the current [`Node`] in `nodes`.  Calls [`Node::next_node`] internally to
    /// track node order.
    pub fn next_node_inner(&mut self) -> Option<Uuid> {
        if let Some(node) = self.nodes.get_mut(&self.current_node()) {
            Some(node.next_node())
        } else {
            None
        }
    }

    /// Moves focus to the previous child node of the current [`Node`] in `nodes`.  Calls [`Node::previous_node`] internally
    /// to track node order.
    pub fn previous_node_inner(&mut self) -> Option<Uuid> {
        if let Some(node) = self.nodes.get_mut(&self.current_node()) {
            Some(node.previous_node())
        } else {
            None
        }
    }

    /// Returns the [`Uuid`] of the active [`Leaf`] in focus, if present.
    pub fn current_leaf(&self) -> Option<Uuid> {
        if let Some(node) = self.nodes.get(&self.current_node()) {
            Some(node.current_leaf())
        } else {
            None
        }
    }

    /// Returns the [`egui::Id`] of the active [`Leaf`] in focus, if present.
    pub fn current_leaf_id(&self) -> Option<egui::Id> {
        if let Some(uuid) = self.current_leaf() {
            if let Some(leaf) = self.leaves.get(&uuid) {
                Some(leaf.id)
            } else {
                unreachable!()
            }
        } else {
            tracing::info!("Leaf not found.");
            None
        }
    }

    /// Advances focus to the next [`Leaf`] in `leaves`.
    pub fn next_leaf(&mut self) -> Option<Uuid> {
        if let Some(node) = self.nodes.get_mut(&self.current_node()) {
            Some(node.next_leaf())
        } else {
            None
        }
    }

    /// Move focus to the previous ['Leaf'] in `leaves`.
    pub fn previous_leaf(&mut self) -> Option<Uuid> {
        if let Some(node) = self.nodes.get_mut(&self.current_node()) {
            Some(node.previous_leaf())
        } else {
            None
        }
    }

    /// Sets the `select` field to the current [`Leaf`] in focus.
    pub fn select_current(&mut self) {
        if let Some(leaf_id) = self.current_leaf() {
            if let Some(leaf) = self.leaves.get(&leaf_id) {
                tracing::info!("Setting select to {:#?}", leaf.id);
                self.select = Some(leaf.id);
            }
        }
    }

    /// Sets the `select` field to the next [`Leaf`] in 'leaves'.
    pub fn select_next(&mut self) {
        if let Some(leaf_id) = self.next_leaf() {
            if let Some(leaf) = self.leaves.get(&leaf_id) {
                tracing::info!("Setting select to {:#?}", leaf.id);
                self.select = Some(leaf.id);
            }
        }
    }

    /// Sets the `select` field to the previous [`Leaf`] in 'leaves'.
    pub fn select_previous(&mut self) {
        if let Some(leaf_id) = self.previous_leaf() {
            if let Some(leaf) = self.leaves.get(&leaf_id) {
                tracing::info!("Setting select to {:#?}", leaf.id);
                self.select = Some(leaf.id);
            }
        }
    }

    /// Sets the `select` field to the next [`Node`] in 'nodes'.
    pub fn select_next_node(&mut self) {
        let _ = self.next_node();
        self.select_current();
    }

    /// Sets the `select` field to the previous [`Node`] in 'nodes'.
    pub fn select_previous_node(&mut self) {
        let _ = self.previous_node();
        self.select_current();
    }

    /// Sets the `select` field to the next window in 'windows'.
    pub fn select_next_window(&mut self) {
        let _ = self.next_window();
        let _ = self.current_node();
        let _ = self.current_leaf();
        self.select_current();
    }

    /// Sets the `select` field to the previous window in 'windows'.
    pub fn select_previous_window(&mut self) {
        let _ = self.previous_window();
        let _ = self.current_node();
        let _ = self.current_leaf();
        self.select_current();
    }

    pub fn select_window(&mut self, id: &Uuid) {
        tracing::info!("Comp id: {}", id);
        let index = self
            .windows
            .iter()
            .enumerate()
            .filter(|(_, v)| *v == id)
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();
        if !index.is_empty() {
            self.window_index = index[0];
        } else {
            tracing::warn!("Window ID not found!");
        }
    }

    pub fn update(&mut self, parent: &mut Tree, child: Tree) {
        let check = match self.flags.get(&self.current_window()) {
            Some(&bool) => Some(bool),
            None => {
                if self.flags.is_empty() {
                    tracing::info!("Tree data not present.");
                    tracing::info!("Grafting child to parent tree.");
                    parent.graft(child.clone());
                }
                None
            }
        };
        if let Some(loaded) = check {
            if loaded == false {
                self.graft(child);
                tracing::info!("Grafting child directly to tree.");
                if let Some(value) = self.flags.get_mut(&self.current_window()) {
                    tracing::info!("Marking load widget as loaded.");
                    *value = true;
                    tracing::info!("{:#?}", self);
                }
            }
        }
    }
}

/// The `Node` struct takes ['Leaf'] and [`Node`] types as children, and may claim a [`Node`] as a
/// parent.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// The `id` field uses a [`Uuid`] as the internal unique identifier for the library.
    pub id: Uuid,
    /// The `parent` field holds the [`Uuid`] of the parent [`Node`].
    pub parent: Option<Uuid>,
    /// The vector in the `nodes` field contains the [`Uuid`] of each child [`Node`] claiming this node
    /// as a parent.
    pub nodes: Vec<Uuid>,
    /// The vector in the `leaves` field contains the [`Uuid`] of each child [`Leaf`] claiming this
    /// node as a parent.
    pub leaves: Vec<Uuid>,
    /// The `window` field contains the [`Uuid`] of the associated window.
    pub window: Option<Uuid>,
    // Index of the current focus child [`Node`].
    node_index: usize,
    // Index of the current focus child ['Leaf'].
    leaf_index: usize,
}

impl Node {
    /// Creates a new instance of [`Node`].
    pub fn new() -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            ..Default::default()
        }
    }

    /// Creates an empty `Node` and inserts it into the [`Tree`] specified in parameter `tree`.
    /// Returns the [`Uuid`] for the new node.  Populate the new node by using the [`Uuid`] to obtain a mutable reference to
    /// the node.
    pub fn with_tree(tree: &mut Tree) -> Uuid {
        let id = Uuid::new_v4();
        let node = Self {
            id,
            ..Default::default()
        };
        tree.nodes.insert(id, node);
        id
    }

    /// Uses the `node_id` and `leaf_id` to obtain mutable references to the node and leaf
    /// respectively within the [`Tree`] parameter `tree`.  Adds the given leaf to the child leaves
    /// of the given node.
    pub fn with_leaf(node_id: Uuid, leaf_id: Uuid, tree: &mut Tree) {
        let leaf = tree.leaves.get_mut(&leaf_id);
        let node = tree.nodes.get_mut(&node_id);
        if let Some(l) = leaf {
            l.parent = Some(node_id);
            if let Some(n) = node {
                n.leaves.push(l.leaf_id);
            }
        }
    }

    /// The `with_branch` method adds the [`Node`] from parameter `node` to the children nodes of
    /// self.  The `node` parameter must be mutable to set the `parent` field of `node` to the
    /// [`Uuid`] of self.
    pub fn with_branch(&mut self, node: &mut Node) {
        node.parent = Some(self.id.to_owned());
        self.nodes.push(node.id);
    }

    /// The `with_window` method sets the associated window of the node to the [`Uuid`] in the
    /// window parameter.
    pub fn with_window(&mut self, window: Uuid) {
        self.window = Some(window);
    }

    /// The `current_leaf` method returns the [`Uuid`] of the [`Leaf`] in `leaves` at the `leaf_index`.
    pub fn current_leaf(&self) -> Uuid {
        let id = self.leaves[self.leaf_index];
        info!("Current leaf is {}", id);
        id
    }

    pub fn next_leaf(&mut self) -> Uuid {
        if self.leaf_index == (self.leaves.len() - 1) {
            self.leaf_index = 0;
        } else {
            self.leaf_index += 1;
        }
        info!("Leaf index is {}", self.leaf_index);
        let id = self.leaves[self.leaf_index];
        info!("Next leaf is {}", id);
        id
    }

    pub fn previous_leaf(&mut self) -> Uuid {
        if self.leaf_index == 0 {
            self.leaf_index = self.leaves.len() - 1;
        } else {
            self.leaf_index -= 1;
        }
        let id = self.leaves[self.leaf_index];
        info!("Previous leaf is {}", id);
        id
    }

    pub fn current_node(&self) -> Uuid {
        let id = self.nodes[self.node_index];
        info!("Current node is {}", id);
        id
    }

    pub fn next_node(&mut self) -> Uuid {
        if self.node_index + 1 > self.nodes.len() - 1 {
            self.node_index += 1;
        } else {
            self.node_index = 0;
        }
        let id = self.nodes[self.node_index];
        info!("Next node is {}", id);
        id
    }

    pub fn previous_node(&mut self) -> Uuid {
        if self.node_index == 0 {
            self.node_index = self.nodes.len() - 1;
        } else {
            self.node_index -= 1;
        }
        let id = self.nodes[self.node_index];
        info!("Previous node is {}", id);
        id
    }
}

/// The `Leaf` struct represent focus points that have corresponding visual elements in the user
/// interface.  Create a [`Leaf`] from an [`egui::Id`] and bind it to a [`Node`] using
/// [`Node::with_leaf`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Leaf {
    /// The `id` field is the [`egui::Id`] of the visual element.
    pub id: Id,
    /// The `leaf_id` field is the [`Uuid`] tracked by the [`Tree`].
    pub leaf_id: Uuid,
    /// The `parent` field is the [`Uuid`] of the parent [`Node`].
    pub parent: Option<Uuid>,
}

impl Leaf {
    pub fn from_id(id: Id, tree: &mut Tree) -> Uuid {
        // Creates a new internal id.
        let leaf_id = Uuid::new_v4();
        // Default to None for parent node.
        let leaf = Self {
            id,
            leaf_id,
            parent: None,
        };
        // Attach to focus tree.
        tree.leaves.insert(leaf_id, leaf);
        leaf_id
    }
}
