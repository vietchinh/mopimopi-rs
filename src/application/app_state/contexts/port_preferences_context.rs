//! The port-only preferences (see `domain::settings::PortPreferences`): a store of their own, saved on every change.

use crate::domain::settings::PortPreferences;
use crate::infrastructure::browser_storage::{load_port_preferences, save_port_preferences};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct PortPreferencesContext {
    pub preferences: Signal<PortPreferences>,
}

impl PortPreferencesContext {
    /// Loads the saved preferences and provides them to everything below. A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let preferences = use_signal(load_port_preferences);
        use_context_provider(|| Self { preferences })
    }

    /// Replaces the preferences and saves them. Nothing here touches the settings file or its storage key.
    pub fn set(&self, preferences: PortPreferences) {
        save_port_preferences(&preferences);
        let mut signal = self.preferences;
        signal.set(preferences);
    }
}
