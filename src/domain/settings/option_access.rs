//! Reading settings. Every reader takes the key used in `defaults.json` / `l.json`.

use super::Settings;
use serde_json::Value;

impl Settings {
    /// Raw value of an option (switches, choices, text); null when there is no such option.
    pub fn option_value(&self, key: &str) -> Value {
        self.file.get_option(key).map_or(Value::Null, |value| value.to_value())
    }

    /// True when the option is switched on (JavaScript truthiness).
    pub fn option_enabled(&self, key: &str) -> bool {
        self.file.get_option(key).is_some_and(|value| value.is_truthy())
    }

    pub fn option_number(&self, key: &str) -> f64 {
        self.file.get_option(key).map_or(0.0, |value| value.as_number())
    }

    pub fn option_text(&self, key: &str) -> String {
        self.file.get_option(key).map(|value| value.as_text()).unwrap_or_default()
    }

    /// UI language code: KR, JP, EN, FR, DE or CN.
    pub fn language_code(&self) -> String {
        let code = self.option_text("Lang");
        if code.is_empty() { "EN".into() } else { code }
    }

    /// Value of a slider setting (opacity in percent, sizes in tenths of a rem, ...).
    pub fn slider_value(&self, key: &str) -> f64 {
        self.file.get_range(key).unwrap_or(0.0)
    }

    /// Colour as six hex digits without `#` ("03A9F4"); black when the key is unknown.
    pub fn color_hex(&self, key: &str) -> String {
        self.file.get_color(key).unwrap_or("000000").to_string()
    }

    /// Saved action abbreviations: (long action name, short name).
    pub fn action_abbreviations(&self) -> Vec<(String, String)> {
        self.file.aliases.iter().map(|(action, short)| (action.clone(), short.clone())).collect()
    }
}
