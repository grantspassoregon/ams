//! The `act` module encapsulates the event handling model for the application by classifying
//! application functions as variants of the `Act` enum.
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

/// The `Act` enum delineates the types of application functions that are accessible to the user.
/// The `command` module maps keyboard inputs to specific variants of the `Act` enum as an action
/// handling model.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, EnumIter, Deserialize, Serialize)]
pub enum Act {
    /// Event handlers for the `winit` library.
    App(AppAct),
    /// Event handlers for the `egui` library.
    Egui(EguiAct),
    /// Event handlers for named keys.
    Named(NamedAct),
    /// A no-op action.
    #[default]
    Be,
}

impl Act {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn idx(&self) -> usize {
        match self {
            Self::App(act) => act.idx(),
            Self::Egui(act) => act.idx() + 100,
            Self::Named(act) => act.idx() + 200,
            Self::Be => 999,
        }
    }
}

impl PartialOrd for Act {
    fn partial_cmp(&self, other: &Act) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Act {
    fn cmp(&self, other: &Act) -> std::cmp::Ordering {
        let self_id = self.idx();
        let other_id = other.idx();
        self_id.cmp(&other_id)
    }
}

impl std::string::ToString for Act {
    fn to_string(&self) -> String {
        match self {
            Self::App(act) => act.to_string(),
            Self::Egui(act) => act.to_string(),
            Self::Named(act) => act.to_string(),
            Self::Be => "Be".to_string(),
        }
    }
}

impl std::str::FromStr for Act {
    type Err = aid::prelude::Bandage;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(act) = AppAct::from_str(s) {
            Ok(Self::App(act))
        } else if let Ok(act) = EguiAct::from_str(s) {
            Ok(Self::Egui(act))
        } else if let Ok(act) = NamedAct::from_str(s) {
            Ok(Self::Named(act))
        } else if &s.to_lowercase() == "be" {
            Ok(Self::Be)
        } else {
            Err(aid::prelude::Bandage::Hint("Undefined act.".to_string()))
        }
    }
}

impl From<AppAct> for Act {
    fn from(act: AppAct) -> Self {
        match act {
            AppAct::Be => Self::Be,
            other => Self::App(other),
        }
    }
}

impl From<&AppAct> for Act {
    fn from(act: &AppAct) -> Self {
        match act {
            AppAct::Be => Self::Be,
            other => Self::App(*other),
        }
    }
}

impl From<EguiAct> for Act {
    fn from(act: EguiAct) -> Self {
        match act {
            EguiAct::Be => Self::Be,
            other => Self::Egui(other),
        }
    }
}

impl From<&EguiAct> for Act {
    fn from(act: &EguiAct) -> Self {
        match act {
            EguiAct::Be => Self::Be,
            other => Self::Egui(*other),
        }
    }
}

impl From<NamedAct> for Act {
    fn from(act: NamedAct) -> Self {
        match act {
            NamedAct::Be => Self::Be,
            other => Self::Named(other),
        }
    }
}

impl From<&NamedAct> for Act {
    fn from(act: &NamedAct) -> Self {
        match act {
            NamedAct::Be => Self::Be,
            other => Self::Named(*other),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, EnumIter, Deserialize, Serialize)]
pub enum AppAct {
    Help,
    Menu,
    Decorations,
    Fullscreen,
    Maximize,
    Minimize,
    #[default]
    Be,
}

impl AppAct {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn idx(&self) -> usize {
        match self {
            Self::Help => 0,
            Self::Menu => 1,
            Self::Decorations => 2,
            Self::Fullscreen => 3,
            Self::Maximize => 4,
            Self::Minimize => 5,
            Self::Be => 6,
        }
    }
}

impl PartialOrd for AppAct {
    fn partial_cmp(&self, other: &AppAct) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AppAct {
    fn cmp(&self, other: &AppAct) -> std::cmp::Ordering {
        let self_id = self.idx();
        let other_id = other.idx();
        self_id.cmp(&other_id)
    }
}

impl std::string::ToString for AppAct {
    fn to_string(&self) -> String {
        let str = match self {
            Self::Help => "Help",
            Self::Menu => "Menu",
            Self::Decorations => "Decorations",
            Self::Fullscreen => "Fullscreen",
            Self::Maximize => "Maximize",
            Self::Minimize => "Minimize",
            Self::Be => "Be",
        };
        str.to_string()
    }
}

impl std::str::FromStr for AppAct {
    type Err = aid::prelude::Bandage;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "help" => Ok(Self::Help),
            "menu" => Ok(Self::Menu),
            "decorations" => Ok(Self::Decorations),
            "fullscreen" => Ok(Self::Fullscreen),
            "maximize" => Ok(Self::Maximize),
            "minimize" => Ok(Self::Minimize),
            "be" => Ok(Self::Be),
            _ => Err(aid::prelude::Bandage::Hint("Undefined act.".to_string())),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, EnumIter, Deserialize, Serialize)]
pub enum EguiAct {
    Right,
    Left,
    Up,
    Down,
    Next,
    Previous,
    NextWindow,
    PreviousWindow,
    NextRow,
    PreviousRow,
    #[default]
    Be,
}

impl EguiAct {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn idx(&self) -> usize {
        match self {
            Self::Right => 0,
            Self::Left => 1,
            Self::Up => 2,
            Self::Down => 3,
            Self::Next => 4,
            Self::Previous => 5,
            Self::NextWindow => 6,
            Self::PreviousWindow => 7,
            Self::NextRow => 8,
            Self::PreviousRow => 9,
            Self::Be => 10,
        }
    }
}

impl PartialOrd for EguiAct {
    fn partial_cmp(&self, other: &EguiAct) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EguiAct {
    fn cmp(&self, other: &EguiAct) -> std::cmp::Ordering {
        let self_id = self.idx();
        let other_id = other.idx();
        self_id.cmp(&other_id)
    }
}

impl std::string::ToString for EguiAct {
    fn to_string(&self) -> String {
        let str = match self {
            Self::Right => "Right",
            Self::Left => "Left",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Next => "Next",
            Self::Previous => "Previous",
            Self::NextWindow => "Next Window",
            Self::PreviousWindow => "Previous Window",
            Self::NextRow => "Next Row",
            Self::PreviousRow => "Previous Row",
            Self::Be => "Be",
        };
        str.to_string()
    }
}

impl std::str::FromStr for EguiAct {
    type Err = aid::prelude::Bandage;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "right" => Ok(Self::Right),
            "left" => Ok(Self::Left),
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "next" => Ok(Self::Next),
            "previous" => Ok(Self::Previous),
            "next_window" => Ok(Self::NextWindow),
            "previous_window" => Ok(Self::PreviousWindow),
            "next_row" => Ok(Self::NextRow),
            "previous_row" => Ok(Self::PreviousRow),
            "be" => Ok(Self::Be),
            _ => Err(aid::prelude::Bandage::Hint("Undefined act.".to_string())),
        }
    }
}

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    // PartialOrd,
    Eq,
    // Ord,
    Hash,
    EnumIter,
    Deserialize,
    Serialize,
)]
pub enum NamedAct {
    Enter,
    Escape,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    #[default]
    Be,
}

impl NamedAct {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cmd(&self) -> String {
        let value = match self {
            Self::Enter => "enter",
            Self::Escape => "escape",
            Self::ArrowUp => "arrow_up",
            Self::ArrowDown => "arrow_down",
            Self::ArrowLeft => "arrow_left",
            Self::ArrowRight => "arrow_right",
            Self::Be => "be",
        };
        value.to_owned()
    }

    pub fn idx(&self) -> usize {
        match self {
            Self::Enter => 0,
            Self::Escape => 1,
            Self::ArrowUp => 2,
            Self::ArrowDown => 3,
            Self::ArrowLeft => 4,
            Self::ArrowRight => 5,
            Self::Be => 6,
        }
    }
}

impl PartialOrd for NamedAct {
    fn partial_cmp(&self, other: &NamedAct) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NamedAct {
    fn cmp(&self, other: &NamedAct) -> std::cmp::Ordering {
        let self_id = self.idx();
        let other_id = other.idx();
        self_id.cmp(&other_id)
    }
}

impl From<&winit::keyboard::NamedKey> for NamedAct {
    fn from(named: &winit::keyboard::NamedKey) -> Self {
        match named {
            winit::keyboard::NamedKey::Enter => Self::Enter,
            winit::keyboard::NamedKey::Escape => Self::Escape,
            winit::keyboard::NamedKey::ArrowLeft => Self::ArrowLeft,
            winit::keyboard::NamedKey::ArrowRight => Self::ArrowRight,
            winit::keyboard::NamedKey::ArrowUp => Self::ArrowUp,
            winit::keyboard::NamedKey::ArrowDown => Self::ArrowDown,
            _ => Self::Be,
        }
    }
}

impl From<&winit::keyboard::Key> for NamedAct {
    fn from(named: &winit::keyboard::Key) -> Self {
        match named {
            winit::keyboard::Key::Named(k) => Self::from(k),
            _ => Self::Be,
        }
    }
}

impl std::string::ToString for NamedAct {
    fn to_string(&self) -> String {
        let str = match self {
            Self::Enter => "Enter",
            Self::Escape => "Escape",
            Self::ArrowLeft => "Arrow Left",
            Self::ArrowRight => "Arrow Right",
            Self::ArrowUp => "Arrow Up",
            Self::ArrowDown => "Arrow Down",
            Self::Be => "Be",
        };
        str.to_string()
    }
}

impl std::str::FromStr for NamedAct {
    type Err = aid::prelude::Bandage;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enter" => Ok(Self::Enter),
            "escape" => Ok(Self::Escape),
            "arrow_left" => Ok(Self::ArrowLeft),
            "arrow_right" => Ok(Self::ArrowRight),
            "arrow_up" => Ok(Self::ArrowUp),
            "arrow_down" => Ok(Self::ArrowDown),
            "be" => Ok(Self::Be),
            _ => Err(aid::prelude::Bandage::Hint("Undefined act.".to_string())),
        }
    }
}
