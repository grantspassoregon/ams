use crate::prelude::Action;
use winit::keyboard::ModifiersState;

pub struct Binding<T: Eq> {
    pub trigger: T,
    pub mods: ModifiersState,
    pub action: Action,
}

impl<T: Eq> Binding<T> {
    pub const fn new(trigger: T, mods: ModifiersState, action: Action) -> Self {
        Self {
            trigger,
            mods,
            action,
        }
    }

    pub fn is_triggered_by(&self, trigger: &T, mods: &ModifiersState) -> bool {
        &self.trigger == trigger && &self.mods == mods
    }

    pub fn modifiers(&self) -> String {
        let mut mods_line = String::new();
        // Always add + since it's printed as a part of the bindings.
        for (modifier, desc) in [
            (ModifiersState::SUPER, "Super+"),
            (ModifiersState::ALT, "Alt+"),
            (ModifiersState::CONTROL, "Ctrl+"),
            (ModifiersState::SHIFT, "Shift+"),
        ] {
            if !self.mods.contains(modifier) {
                continue;
            }

            mods_line.push_str(desc);
        }
        mods_line
    }
}
