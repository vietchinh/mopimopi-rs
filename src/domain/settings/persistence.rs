//! Loading, normalising and saving settings.

use super::browser_storage::{read_local_storage, write_local_storage};
use super::default_settings::default_settings_document;
use super::language_detection::detect_language_code;
use super::user_settings::*;
use super::Settings;
use serde_json::{json, Value};

/// `localStorage` key of the live settings (same key as the original overlay).
pub const SETTINGS_STORAGE_KEY: &str = "Mopi2_HAERU";
/// `localStorage` key of the backup made from the Tools page.
pub const BACKUP_STORAGE_KEY: &str = "backup";

/// Raid mode default that the original changed from 10 to 14 players in a later version.
const OUTDATED_RAID_MODE_THRESHOLD: i64 = 10;
const CURRENT_RAID_MODE_THRESHOLD: i64 = 14;

impl Settings {
    pub fn defaults() -> Settings {
        Settings { json_document: default_settings_document() }
    }

    /// Reads settings from `localStorage`, falling back to defaults with a language matching the
    /// browser. Options added by newer versions are filled in from the defaults.
    pub fn load_from_browser() -> Settings {
        let defaults = default_settings_document();
        let stored = read_local_storage(SETTINGS_STORAGE_KEY)
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .filter(Value::is_object);
        let mut document = stored.unwrap_or_else(|| {
            let mut first_run = defaults.clone();
            first_run[OPTIONS_SECTION]["Lang"] = json!(detect_language_code());
            first_run
        });
        normalize_document(&mut document, &defaults);
        Settings { json_document: document }
    }

    /// Parses settings from text (used to restore a backup). `None` if it is not a settings object.
    pub fn from_json_text(text: &str) -> Option<Settings> {
        let mut document = serde_json::from_str::<Value>(text).ok().filter(Value::is_object)?;
        normalize_document(&mut document, &default_settings_document());
        Some(Settings { json_document: document })
    }

    pub fn to_json_text(&self) -> String {
        serde_json::to_string(&self.json_document).unwrap_or_default()
    }

    pub fn save_to_browser(&self) {
        write_local_storage(SETTINGS_STORAGE_KEY, &self.to_json_text());
    }

    /// Re-applies normalisation (after importing shared options).
    pub(super) fn normalize(&mut self) {
        normalize_document(&mut self.json_document, &default_settings_document());
    }
}

/// Brings a possibly old or partial settings document up to the current shape.
fn normalize_document(document: &mut Value, defaults: &Value) {
    for section in [OPTIONS_SECTION, COLORS_SECTION, SLIDERS_SECTION, ABBREVIATIONS_SECTION] {
        if !document.get(section).map(Value::is_object).unwrap_or(false) {
            document[section] = defaults[section].clone();
        }
    }
    for section in [OPTIONS_SECTION, COLORS_SECTION, SLIDERS_SECTION] {
        let default_entries = defaults[section].as_object().cloned().unwrap_or_default();
        let entries = document[section].as_object_mut().expect("section was made an object above");
        for (key, default_value) in default_entries {
            entries.entry(key).or_insert(default_value);
        }
        entries.retain(|key, _| !key.contains("Cell")); // removed in an old version of the original
    }
    let has_column_widths = document.pointer("/ColData/Class/width").map(|width| !width.is_null()).unwrap_or(false);
    let has_column_order = document.get(COLUMN_ORDER_SECTION).map(Value::is_object).unwrap_or(false);
    if !has_column_widths || !has_column_order {
        document[COLUMN_DEFINITIONS_SECTION] = defaults[COLUMN_DEFINITIONS_SECTION].clone();
        document[COLUMN_ORDER_SECTION] = defaults[COLUMN_ORDER_SECTION].clone();
    }
    if document[OPTIONS_SECTION]["view24_Number"] == json!(OUTDATED_RAID_MODE_THRESHOLD) {
        document[OPTIONS_SECTION]["view24_Number"] = json!(CURRENT_RAID_MODE_THRESHOLD);
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/persistence.rs"]
mod tests;
