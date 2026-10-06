//! Loading and saving settings.

use super::browser_storage::{read_local_storage, write_local_storage};
use super::import::{defaults, import};
use super::language_detection::detect_language_code;
use super::settings_file::OptionValue;
use super::Settings;

/// `localStorage` key of the live settings (same key as the original overlay).
pub const SETTINGS_STORAGE_KEY: &str = "Mopi2_HAERU";
/// `localStorage` key of the backup made from the Tools page.
pub const BACKUP_STORAGE_KEY: &str = "backup";

impl Settings {
    pub fn defaults() -> Settings {
        Settings { file: defaults().clone() }
    }

    /// Reads settings from `localStorage`, falling back to defaults with a language matching the
    /// browser. Options added by newer versions are filled in from the defaults.
    pub fn load_from_browser() -> Settings {
        let stored = read_local_storage(SETTINGS_STORAGE_KEY).and_then(|text| match import(&text) {
            Ok(file) => Some(file),
            Err(error) => {
                tracing::warn!("the saved settings were not used ({error:?}); starting from the defaults");
                None
            }
        });
        match stored {
            Some(file) => Settings { file },
            None => {
                let mut first_run = Settings::defaults();
                first_run.file.set_option("Lang", OptionValue::Text(detect_language_code().to_string()));
                first_run
            }
        }
    }

    /// Parses settings from text (used to restore a backup). `None` if it is not a settings object.
    pub fn from_json_text(text: &str) -> Option<Settings> {
        import(text).ok().map(|file| Settings { file })
    }

    pub fn to_json_text(&self) -> String {
        serde_json::to_string(&self.file).unwrap_or_default()
    }

    pub fn save_to_browser(&self) {
        write_local_storage(SETTINGS_STORAGE_KEY, &self.to_json_text());
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/persistence.rs"]
mod tests;
