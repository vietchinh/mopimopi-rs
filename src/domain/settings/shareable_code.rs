//! "Custom UI Data": export the look of the overlay as text and import somebody else's.

use super::import::import_value;
use super::settings_file::{COLORS_SECTION, OPTIONS_SECTION, RANGES_SECTION};
use super::user_settings::SHAREABLE_SECTIONS;
use super::Settings;
use serde_json::Value;

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
        // `shift_remove`, not `remove`: with order preservation on, `remove` swaps the last entry into the gap, which reorders the
        // exported code (the original deletes keys in place).
        let mut options = self.file.options.clone();
        for key in NOT_SHARED_OPTIONS {
            options.shift_remove(key);
        }
        let mut ranges = self.file.ranges.clone();
        for key in NOT_SHARED_SLIDERS {
            ranges.shift_remove(key);
        }
        serde_json::json!({ OPTIONS_SECTION: options, COLORS_SECTION: self.file.colors, RANGES_SECTION: ranges }).to_string()
    }

    /// Applies a shared code. Fails (changing nothing) when the text is not a settings object.
    pub fn import_shareable_code(&mut self, code: &str) -> Result<(), InvalidShareableCode> {
        let parsed: Value = serde_json::from_str(code).map_err(|_| InvalidShareableCode)?;
        let sections = parsed.as_object().ok_or(InvalidShareableCode)?;
        // The code's values go on top of the current settings, and the result is loaded like any other file: brought up to date, repaired, typed.
        let mut document = serde_json::to_value(&self.file).map_err(|_| InvalidShareableCode)?;
        for section in SHAREABLE_SECTIONS {
            if let Some(entries) = sections.get(section).and_then(Value::as_object) {
                for (key, value) in entries {
                    document[section][key] = value.clone();
                }
            }
        }
        self.file = import_value(document).map_err(|_| InvalidShareableCode)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/shareable_code.rs"]
mod tests;
