//! Table columns: their definitions, which are enabled per table, and their order.

use super::json_coercion::is_truthy;
use super::user_settings::*;
use super::Settings;
use serde_json::{json, Map, Value};

impl Settings {
    /// All known columns with their definition (title, width, padding, alignment, table flags).
    pub fn column_definitions(&self) -> &Map<String, Value> {
        static EMPTY: std::sync::OnceLock<Map<String, Value>> = std::sync::OnceLock::new();
        self.json_document[COLUMN_DEFINITIONS_SECTION].as_object().unwrap_or_else(|| EMPTY.get_or_init(Map::new))
    }

    pub fn column_text(&self, column: &str, field: &str) -> String {
        match self.json_document[COLUMN_DEFINITIONS_SECTION][column].get(field) {
            Some(Value::String(text)) => text.clone(),
            Some(Value::Number(number)) => number.to_string(),
            _ => String::new(),
        }
    }

    pub fn set_column_field(&mut self, column: &str, field: &str, value: Value) {
        self.json_document[COLUMN_DEFINITIONS_SECTION][column][field] = value;
    }

    /// Whether `column` is switched on for the table (`"DPS"` or `"HPS"`).
    pub fn column_enabled_in_table(&self, column: &str, table_label: &str) -> bool {
        is_truthy(&self.json_document[COLUMN_DEFINITIONS_SECTION][column][table_label])
    }

    /// Columns of a table in display order.
    pub fn column_order(&self, table_label: &str) -> Vec<String> {
        self.json_document[COLUMN_ORDER_SECTION][table_label]
            .as_array()
            .map(|names| names.iter().filter_map(|name| name.as_str().map(String::from)).collect())
            .unwrap_or_default()
    }

    fn set_column_order(&mut self, table_label: &str, order: Vec<String>) {
        self.json_document[COLUMN_ORDER_SECTION][table_label] = json!(order);
    }

    /// Switches a column on or off for a table, keeping the display order in sync.
    pub fn set_column_enabled(&mut self, column: &str, table_label: &str, enabled: bool) {
        self.set_column_field(column, table_label, json!(enabled as i32));
        let mut order = self.column_order(table_label);
        if enabled {
            if !order.iter().any(|name| name == column) {
                order.push(column.to_string());
            }
        } else {
            order.retain(|name| name != column);
        }
        self.set_column_order(table_label, order);
    }

    /// Swaps a column with its neighbour (`move_up`: towards the front).
    pub fn move_column(&mut self, column: &str, table_label: &str, move_up: bool) {
        let mut order = self.column_order(table_label);
        let Some(index) = order.iter().position(|name| name == column) else { return };
        let neighbour = if move_up { index.checked_sub(1) } else { Some(index + 1) };
        if let Some(neighbour) = neighbour.filter(|&position| position < order.len()) {
            order.swap(index, neighbour);
            self.set_column_order(table_label, order);
        }
    }
}
