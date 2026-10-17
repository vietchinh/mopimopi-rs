//! The settings screens: which page and tab are open, and the sample tables above the rows.

use dioxus::prelude::*;

/// Which settings page and tab are open.
#[derive(Clone, PartialEq, Debug)]
pub struct SettingsLocation {
    pub page: String,
    pub tab: Option<String>,
}

impl SettingsLocation {
    pub fn top_level() -> Self {
        SettingsLocation { page: "Settings".into(), tab: None }
    }
}

#[derive(Clone, Copy)]
pub struct SettingsScreenContext {
    pub settings_location: Signal<SettingsLocation>,
    /// Sample tables on settings pages.
    pub settings_preview_enabled: Signal<bool>,
    pub settings_preview_raid_mode: Signal<bool>,
}

impl SettingsScreenContext {
    /// A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let context = Self {
            settings_location: use_signal(SettingsLocation::top_level),
            settings_preview_enabled: use_signal(|| false),
            settings_preview_raid_mode: use_signal(|| false),
        };
        use_context_provider(|| context)
    }
}
