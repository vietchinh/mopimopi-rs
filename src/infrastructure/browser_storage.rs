//! The browser's `localStorage`: where the settings and their backup are kept.
//!
//! This is the only place that touches storage or the browser language; the domain's `Settings` only turns
//! text into settings and back (DDD: repository / infrastructure kept out of the domain). Failures (private
//! mode, quota) are ignored: settings then simply are not remembered.

use crate::domain::settings::{language_code_for_browser_language, PortPreferences, Settings};

/// `localStorage` key of the live settings (same key as the original overlay).
const SETTINGS_STORAGE_KEY: &str = "Mopi2_HAERU";
/// `localStorage` key of the port-only preferences: a store of its own, so the settings file and the original's key are untouched.
const PORT_PREFERENCES_STORAGE_KEY: &str = "MopiMopi_port_preferences";
/// `localStorage` key of the backup made from the Tools page.
const BACKUP_STORAGE_KEY: &str = "backup";

fn read_local_storage(key: &str) -> Option<String> {
    web_sys::window()?.local_storage().ok()??.get_item(key).ok()?
}

fn write_local_storage(key: &str, value: &str) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|window| window.local_storage()) {
        let _ = storage.set_item(key, value);
    }
}

fn browser_language() -> String {
    web_sys::window().and_then(|window| window.navigator().language()).unwrap_or_default().to_lowercase()
}

/// The saved settings, or the defaults with a language matching the browser on a first run.
pub fn load_settings() -> Settings {
    let stored = read_local_storage(SETTINGS_STORAGE_KEY);
    Settings::from_stored(stored.as_deref(), language_code_for_browser_language(&browser_language()))
}

pub fn save_settings(settings: &Settings) {
    write_local_storage(SETTINGS_STORAGE_KEY, &settings.to_json_text());
}

pub fn save_backup(settings: &Settings) {
    write_local_storage(BACKUP_STORAGE_KEY, &settings.to_json_text());
}

/// The backed-up settings, if there is a backup and it is a settings object.
pub fn load_backup() -> Option<Settings> {
    read_local_storage(BACKUP_STORAGE_KEY).and_then(|text| Settings::from_json_text(&text))
}

pub fn load_port_preferences() -> PortPreferences {
    PortPreferences::from_stored(read_local_storage(PORT_PREFERENCES_STORAGE_KEY).as_deref())
}

pub fn save_port_preferences(preferences: &PortPreferences) {
    write_local_storage(PORT_PREFERENCES_STORAGE_KEY, &preferences.to_stored());
}
