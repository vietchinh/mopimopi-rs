//! Changing settings.

use super::json_coercion::{as_number, number_to_json};
use super::user_settings::*;
use super::Settings;
use serde_json::{json, Value};

impl Settings {
    pub fn set_option(&mut self, key: &str, value: Value) {
        self.json_document[OPTIONS_SECTION][key] = value;
    }

    pub fn set_option_enabled(&mut self, key: &str, enabled: bool) {
        self.json_document[OPTIONS_SECTION][key] = json!(enabled as i32);
    }

    /// Sets an option from the text of a picked list entry, keeping the stored type
    /// (a number stays a number, text stays text).
    pub fn set_option_from_text(&mut self, key: &str, text: &str) {
        let is_number_option = self.json_document[OPTIONS_SECTION][key].is_number();
        self.json_document[OPTIONS_SECTION][key] =
            if is_number_option { number_to_json(text.parse().unwrap_or(0.0)) } else { json!(text) };
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
            entries.remove(action_name);
        }
    }

    /// Reads a slider-like number stored in a column definition (`width`, `padding`).
    pub fn column_number(&self, column: &str, field: &str) -> f64 {
        as_number(&self.json_document[COLUMN_DEFINITIONS_SECTION][column][field])
    }
}
