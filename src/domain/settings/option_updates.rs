//! Changing settings.

use super::settings_file::OptionValue;
use super::Settings;
use serde_json::Value;

impl Settings {
    /// Sets an option. A value an option cannot be (null, a list, an object) is ignored.
    pub fn set_option(&mut self, key: &str, value: Value) {
        if let Some(value) = OptionValue::from_value(&value) {
            self.file.set_option(key, value);
        }
    }

    pub fn set_option_enabled(&mut self, key: &str, enabled: bool) {
        self.file.set_option(key, OptionValue::switch(enabled));
    }

    /// Sets an option from the key of a picked list entry. Stored as text whatever the entry looks like,
    /// as the original does (`"dpsType": "1"`): the readers (`option_number`, `option_text`) accept either.
    pub fn set_option_from_text(&mut self, key: &str, text: &str) {
        self.file.set_option(key, OptionValue::Text(text.to_string()));
    }

    pub fn set_slider_value(&mut self, key: &str, value: f64) {
        self.file.set_range(key, value);
    }

    pub fn set_color_hex(&mut self, key: &str, hex_digits: &str) {
        self.file.set_color(key, hex_digits);
    }

    pub fn set_action_abbreviation(&mut self, action_name: &str, short_name: &str) {
        self.file.aliases.insert(action_name.to_string(), short_name.to_string());
    }

    pub fn remove_action_abbreviation(&mut self, action_name: &str) {
        self.file.aliases.shift_remove(action_name); // keeps the remaining abbreviations in order, like the original's `delete`
    }

    /// Reads a slider-like number stored in a column definition (`width`, `padding`).
    pub fn column_number(&self, column: &str, field: &str) -> f64 {
        self.file.columns.get(column).map_or(0.0, |definition| definition.number(field))
    }
}
