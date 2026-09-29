//! Changing settings.

use super::json_coercion::{as_number, number_to_json};
use super::user_settings::{OPTIONS_SECTION, SLIDERS_SECTION, COLORS_SECTION, ABBREVIATIONS_SECTION, COLUMN_DEFINITIONS_SECTION};
use super::Settings;
use serde_json::{json, Value};

impl Settings {
    pub fn set_option(&mut self, key: &str, value: Value) {
        self.json_document[OPTIONS_SECTION][key] = value;
    }

    pub fn set_option_enabled(&mut self, key: &str, enabled: bool) {
        self.json_document[OPTIONS_SECTION][key] = json!(i32::from(enabled));
    }

    /// Sets an option from the key of a picked list entry. Stored as text whatever the entry looks like,
    /// as the original does (`"dpsType": "1"`): the readers (`option_number`, `option_text`) accept either.
    pub fn set_option_from_text(&mut self, key: &str, text: &str) {
        self.json_document[OPTIONS_SECTION][key] = json!(text);
    }

    pub fn set_slider_value(&mut self, key: &str, value: f64) {
        self.json_document[SLIDERS_SECTION][key] = number_to_json(value);
    }

    pub fn set_color_hex(&mut self, key: &str, hex_digits: &str) {
        self.json_document[COLORS_SECTION][key] = json!(hex_digits);
    }

    pub fn set_action_abbreviation(&mut self, action_name: &str, short_name: &str) {
        self.json_document[ABBREVIATIONS_SECTION][action_name] = json!(short_name);
    }

    pub fn remove_action_abbreviation(&mut self, action_name: &str) {
        if let Some(entries) = self.json_document[ABBREVIATIONS_SECTION].as_object_mut() {
            entries.shift_remove(action_name); // keeps the remaining abbreviations in order, like the original's `delete`
        }
    }

    /// Reads a slider-like number stored in a column definition (`width`, `padding`).
    pub fn column_number(&self, column: &str, field: &str) -> f64 {
        as_number(&self.json_document[COLUMN_DEFINITIONS_SECTION][column][field])
    }
}
