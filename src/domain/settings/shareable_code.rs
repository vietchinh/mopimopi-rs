//! "Custom UI Data": export the look of the overlay as text and import somebody else's.

use super::user_settings::*;
use super::Settings;
use serde_json::{Map, Value};

/// Options that are personal or technical and therefore never exported.
const NOT_SHARED_OPTIONS: [&str; 23] = [
    "Lang", "fTime", "fTarget", "fRPS", "fHd", "fBody", "resolution", "overlayBg", "overlayBgImg",
    "overlayBgSize", "overlayBgRepeat", "backupDate", "autoHide", "tooltips", "toast", "keyboard",
    "preview", "preview24", "swap", "hideName", "pets", "view24", "view24_Number",
];
const NOT_SHARED_SLIDERS: [&str; 1] = ["autoHideTime"];

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidShareableCode;

impl Settings {
    pub fn export_shareable_code(&self) -> String {
        let mut shared = Map::new();
        for section in SHAREABLE_SECTIONS {
            let mut entries = self.json_document[section].as_object().cloned().unwrap_or_default();
            let excluded: &[&str] = match section {
                OPTIONS_SECTION => &NOT_SHARED_OPTIONS,
                SLIDERS_SECTION => &NOT_SHARED_SLIDERS,
                _ => &[],
            };
            for key in excluded {
                entries.remove(*key);
            }
            shared.insert(section.to_string(), Value::Object(entries));
        }
        serde_json::to_string(&Value::Object(shared)).unwrap_or_default()
    }

    /// Applies a shared code. Fails (changing nothing) when the text is not a settings object.
    pub fn import_shareable_code(&mut self, code: &str) -> Result<(), InvalidShareableCode> {
        let parsed: Value = serde_json::from_str(code).map_err(|_| InvalidShareableCode)?;
        let sections = parsed.as_object().ok_or(InvalidShareableCode)?;
        for section in SHAREABLE_SECTIONS {
            if let Some(entries) = sections.get(section).and_then(Value::as_object) {
                for (key, value) in entries {
                    self.json_document[section][key] = value.clone();
                }
            }
        }
        self.normalize();
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/shareable_code.rs"]
mod tests;
