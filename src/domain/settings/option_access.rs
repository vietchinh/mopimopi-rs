//! Reading settings. Every reader takes the key used in `defaults.json` / `l.json`.

use super::json_coercion::{as_number, is_truthy};
use super::user_settings::*;
use super::Settings;
use serde_json::Value;

impl Settings {
    /// Raw value of an option (switches, choices, text).
    pub fn option_value(&self, key: &str) -> &Value {
        &self.json_document[OPTIONS_SECTION][key]
    }

    /// True when the option is switched on (JavaScript truthiness).
    pub fn option_enabled(&self, key: &str) -> bool {
        is_truthy(self.option_value(key))
    }

    pub fn option_number(&self, key: &str) -> f64 {
        as_number(self.option_value(key))
    }

    pub fn option_text(&self, key: &str) -> String {
        match self.option_value(key) {
            Value::String(text) => text.clone(),
            Value::Number(number) => number.to_string(),
            _ => String::new(),
        }
    }

    /// UI language code: KR, JP, EN, FR, DE or CN.
    pub fn language_code(&self) -> String {
        let code = self.option_text("Lang");
        if code.is_empty() { "EN".into() } else { code }
    }

    /// Value of a slider setting (opacity in percent, sizes in tenths of a rem, ...).
    pub fn slider_value(&self, key: &str) -> f64 {
        as_number(&self.json_document[SLIDERS_SECTION][key])
    }

    /// Colour as six hex digits without `#` ("03A9F4"); black when the key is unknown.
    pub fn color_hex(&self, key: &str) -> String {
        self.json_document[COLORS_SECTION][key].as_str().unwrap_or("000000").to_string()
    }

    /// Saved action abbreviations: (long action name, short name).
    pub fn action_abbreviations(&self) -> Vec<(String, String)> {
        self.json_document[ABBREVIATIONS_SECTION]
            .as_object()
            .map(|entries| entries.iter().map(|(action, short)| (action.clone(), short.as_str().unwrap_or("").to_string())).collect())
            .unwrap_or_default()
    }
}
