//! Built-in defaults, generated from the original `init.js` (`tools/extract.js`).

use serde_json::Value;
use std::sync::OnceLock;

/// The defaults as the JSON document they are written in, parsed once.
pub(super) fn default_settings_document() -> &'static Value {
    static DEFAULTS: OnceLock<Value> = OnceLock::new();
    DEFAULTS.get_or_init(|| serde_json::from_str(include_str!("../../data/defaults.json")).expect("data/defaults.json is valid JSON"))
}
