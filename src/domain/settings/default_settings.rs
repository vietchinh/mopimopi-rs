//! Built-in defaults, generated from the original `init.js` (`tools/extract.js`).

use serde_json::Value;

pub(super) fn default_settings_document() -> Value {
    serde_json::from_str(include_str!("../../data/defaults.json")).expect("data/defaults.json is valid JSON")
}
