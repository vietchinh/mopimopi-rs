//! Turning JSON text into a `SettingsFile`: the one place that deals with files from other versions.
//!
//! Everything that reads settings goes through here (browser storage, a backup, a shared code applied on top of the current settings), so
//! a file is treated the same wherever it comes from:
//!
//! 1. **Brought up to date**: sections and keys that newer versions added are filled in from the built-in defaults (a key the file has
//!    keeps its place, new ones follow it), the keys an old version of the original called `...Cell...` are dropped, and format changes
//!    of the original are applied (`migrate`).
//! 2. **Repaired**: a value that cannot be what its section holds (a colour that is not text, a size that is not a number, a column that
//!    is missing a field) is replaced by the default for that key, or dropped when the defaults have none. One bad value must not cost
//!    the user every other setting, which is what refusing the whole file would do.
//! 3. **Typed**: what is left is parsed into `SettingsFile`, which keeps whatever this version does not know.

use super::default_settings::default_settings_document;
use super::json_coercion::number_to_json;
use super::settings_file::{
    ColumnData, SettingsFile, ALIASES_SECTION, COLORS_SECTION, COLUMNS_SECTION, OPTIONS_SECTION, ORDER_SECTION, RANGES_SECTION,
};
use serde_json::{json, Value};
use std::sync::OnceLock;

/// Raid mode default that the original changed from 10 to 14 players in a later version.
const OUTDATED_RAID_MODE_THRESHOLD: i64 = 10;
const CURRENT_RAID_MODE_THRESHOLD: i64 = 14;

#[derive(Debug, PartialEq, Eq)]
pub enum ImportError {
    NotJson,
    NotAnObject,
    /// Still not a settings file after the repairs (should not happen; kept so that it can never panic).
    Invalid(String),
}

/// The built-in defaults, parsed once.
pub fn defaults() -> &'static SettingsFile {
    static DEFAULTS: OnceLock<SettingsFile> = OnceLock::new();
    DEFAULTS.get_or_init(|| serde_json::from_value(default_settings_document().clone()).expect("data/defaults.json is a settings file"))
}

/// Settings from JSON text.
pub fn import(text: &str) -> Result<SettingsFile, ImportError> {
    let document: Value = serde_json::from_str(text).map_err(|_| ImportError::NotJson)?;
    if !document.is_object() {
        return Err(ImportError::NotAnObject);
    }
    import_value(document)
}

/// The same, from a document that is already parsed: a file with other values applied on top of it, as applying a shared code does.
pub fn import_value(mut document: Value) -> Result<SettingsFile, ImportError> {
    let defaults = default_settings_document();
    bring_up_to_date(&mut document, defaults);
    migrate(&mut document);
    repair(&mut document, defaults);
    serde_json::from_value(document).map_err(|error| ImportError::Invalid(error.to_string()))
}

/// Fills in what a possibly old or partial document lacks and removes what is obsolete.
fn bring_up_to_date(document: &mut Value, defaults: &Value) {
    for section in [OPTIONS_SECTION, COLORS_SECTION, RANGES_SECTION, ALIASES_SECTION] {
        if !document.get(section).is_some_and(Value::is_object) {
            document[section] = defaults[section].clone();
        }
    }
    for section in [OPTIONS_SECTION, COLORS_SECTION, RANGES_SECTION] {
        let default_entries = defaults[section].as_object().cloned().unwrap_or_default();
        let entries = document[section].as_object_mut().expect("section was made an object above");
        for (key, default_value) in default_entries {
            entries.entry(key).or_insert(default_value);
        }
        entries.retain(|key, _| !key.contains("Cell")); // removed in an old version of the original
    }
    let has_column_widths = document.pointer("/ColData/Class/width").is_some_and(|width| !width.is_null());
    let has_column_order = document.get(ORDER_SECTION).is_some_and(Value::is_object);
    if !has_column_widths || !has_column_order {
        document[COLUMNS_SECTION] = defaults[COLUMNS_SECTION].clone();
        document[ORDER_SECTION] = defaults[ORDER_SECTION].clone();
    }
}

/// Changes the original made to the format of the file, applied to files written before them. Add the next one here.
fn migrate(document: &mut Value) {
    if document[OPTIONS_SECTION]["view24_Number"] == json!(OUTDATED_RAID_MODE_THRESHOLD) {
        document[OPTIONS_SECTION]["view24_Number"] = json!(CURRENT_RAID_MODE_THRESHOLD);
    }
}

/// Makes every value fit what its section holds (see the module docs).
fn repair(document: &mut Value, defaults: &Value) {
    repair_entries(document, defaults, OPTIONS_SECTION, |value| matches!(value, Value::Number(_) | Value::String(_) | Value::Bool(_)).then(|| value.clone()), true);
    repair_entries(document, defaults, COLORS_SECTION, |value| value.is_string().then(|| value.clone()), true);
    repair_entries(document, defaults, RANGES_SECTION, number_of, true);
    repair_entries(document, defaults, ALIASES_SECTION, |value| value.is_string().then(|| value.clone()), false);
    repair_column_order(document, defaults);
    repair_columns(document, defaults);
}

/// A number, or what reads as one: a number written as text, or a switch written as `true` / `false`.
fn number_of(value: &Value) -> Option<Value> {
    match value {
        Value::Number(_) => Some(value.clone()),
        Value::String(text) => text.trim().parse::<f64>().ok().map(number_to_json),
        Value::Bool(flag) => Some(number_to_json(f64::from(i32::from(*flag)))),
        _ => None,
    }
}

/// Every entry of `section` goes through `fix`: kept as it is, replaced by what it returns, or, when it returns nothing, replaced by the
/// default for its key (if `use_default` and there is one) or dropped. Entries stay in their place.
fn repair_entries(document: &mut Value, defaults: &Value, section: &str, fix: impl Fn(&Value) -> Option<Value>, use_default: bool) {
    let Some(entries) = document.get_mut(section).and_then(Value::as_object_mut) else { return };
    let keys: Vec<String> = entries.keys().cloned().collect();
    for key in keys {
        let fixed = fix(&entries[&key]);
        match fixed {
            Some(value) => {
                if let Some(slot) = entries.get_mut(&key) {
                    if *slot != value {
                        *slot = value;
                    }
                }
            }
            None => match use_default.then(|| defaults[section].get(&key).cloned()).flatten() {
                Some(default_value) => {
                    entries.insert(key, default_value);
                }
                None => {
                    entries.shift_remove(&key);
                }
            },
        }
    }
}

/// `Order` holds a list of column names per table.
fn repair_column_order(document: &mut Value, defaults: &Value) {
    let Some(tables) = document.get_mut(ORDER_SECTION).and_then(Value::as_object_mut) else { return };
    let labels: Vec<String> = tables.keys().cloned().collect();
    for label in labels {
        let names: Option<Vec<Value>> = tables[&label].as_array().map(|list| list.iter().filter(|name| name.is_string()).cloned().collect());
        match names {
            Some(names) => {
                tables.insert(label, Value::Array(names));
            }
            None => match defaults[ORDER_SECTION].get(&label).cloned() {
                Some(default_value) => {
                    tables.insert(label, default_value);
                }
                None => {
                    tables.shift_remove(&label);
                }
            },
        }
    }
}

/// Every column must have all its fields; one that is damaged takes the default definition of that column, or is dropped.
fn repair_columns(document: &mut Value, defaults: &Value) {
    let Some(columns) = document.get_mut(COLUMNS_SECTION).and_then(Value::as_object_mut) else { return };
    let names: Vec<String> = columns.keys().cloned().collect();
    for name in names {
        if serde_json::from_value::<ColumnData>(columns[&name].clone()).is_ok() {
            continue;
        }
        match defaults[COLUMNS_SECTION].get(&name).cloned() {
            Some(default_value) => {
                columns.insert(name, default_value);
            }
            None => {
                columns.shift_remove(&name);
            }
        }
    }
    // A column that newer versions added is switched off for a file that does not have it, so its switch in the settings has something to turn on.
    if let Some(default_columns) = defaults[COLUMNS_SECTION].as_object() {
        for (name, definition) in default_columns {
            if !columns.contains_key(name) {
                let mut added = definition.clone();
                added["DPS"] = json!(0);
                added["HPS"] = json!(0);
                columns.insert(name.clone(), added);
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/import.rs"]
mod tests;
