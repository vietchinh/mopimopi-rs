//! The user's settings, shared by every section of the page.

use crate::domain::settings::Settings;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct SettingsContext {
    pub settings: Signal<Settings>,
}

impl SettingsContext {
    /// Loads the saved settings and provides them to everything below. A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let settings = use_signal(Settings::load_from_browser);
        use_context_provider(|| Self { settings })
    }

    /// Applies a change to the settings (they are saved automatically).
    pub fn edit_settings(&self, change: impl FnOnce(&mut Settings)) {
        let mut settings = self.settings;
        change(&mut settings.write());
    }
}
