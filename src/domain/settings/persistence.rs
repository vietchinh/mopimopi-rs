//! Turning settings into text and back (the browser storage itself lives in `infrastructure::browser_storage`).

use super::import::{defaults, import};
use super::settings_file::OptionValue;
use super::Settings;

impl Settings {
    pub fn defaults() -> Settings {
        Settings { file: defaults().clone() }
    }

    /// Settings from the saved text, falling back to defaults with `first_run_language` when there is none or it
    /// is unusable. Options added by newer versions are filled in from the defaults. Pure: the caller reads storage.
    pub fn from_stored(stored_text: Option<&str>, first_run_language: &str) -> Settings {
        let stored = stored_text.and_then(|text| match import(text) {
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
                first_run.file.set_option("Lang", OptionValue::Text(first_run_language.to_string()));
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
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/persistence.rs"]
mod tests;
