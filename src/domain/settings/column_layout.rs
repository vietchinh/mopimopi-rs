//! Table columns: their definitions, which are enabled per table, and their order.

use super::Settings;
use serde_json::{json, Value};

impl Settings {
    /// Columns that are switched on in at least one table.
    pub fn columns_enabled_anywhere(&self) -> Vec<String> {
        self.file.columns.iter().filter(|(_, definition)| definition.is_on("DPS") || definition.is_on("HPS")).map(|(name, _)| name.clone()).collect()
    }

    /// A field of a column's definition as text (title, width, padding, alignment, ...); empty when there is none.
    pub fn column_text(&self, column: &str, field: &str) -> String {
        self.file.columns.get(column).map(|definition| definition.text(field)).unwrap_or_default()
    }

    /// Changes a field of a column's definition. A column the file does not have, or a value that does not fit the field, is ignored.
    pub fn set_column_field(&mut self, column: &str, field: &str, value: Value) {
        if let Some(definition) = self.file.columns.get_mut(column) {
            definition.set(field, &value);
        }
    }

    /// Columns of a table in display order.
    pub fn column_order(&self, table_label: &str) -> Vec<String> {
        self.file.order.0.get(table_label).cloned().unwrap_or_default()
    }

    fn set_column_order(&mut self, table_label: &str, order: &[String]) {
        self.file.order.0.insert(table_label.to_string(), order.to_vec());
    }

    /// Switches a column on or off for a table, keeping the display order in sync.
    pub fn set_column_enabled(&mut self, column: &str, table_label: &str, enabled: bool) {
        self.set_column_field(column, table_label, json!(i32::from(enabled)));
        let mut order = self.column_order(table_label);
        if enabled {
            if !order.iter().any(|name| name == column) {
                order.push(column.to_string());
            }
        } else {
            order.retain(|name| name != column);
        }
        self.set_column_order(table_label, &order);
    }

    /// Swaps a column with its neighbour (`move_up`: towards the front).
    pub fn move_column(&mut self, column: &str, table_label: &str, move_up: bool) {
        let mut order = self.column_order(table_label);
        let Some(index) = order.iter().position(|name| name == column) else { return };
        let neighbour = if move_up { index.checked_sub(1) } else { Some(index + 1) };
        if let Some(neighbour) = neighbour.filter(|&position| position < order.len()) {
            order.swap(index, neighbour);
            self.set_column_order(table_label, &order);
        }
    }
}
