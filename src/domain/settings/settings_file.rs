//! The settings file as a typed value.
//!
//! This is the JSON document the original overlay kept in `localStorage` (and that backups and shared "Custom UI Data" codes carry),
//! one struct per section, read and written by serde. It is the only thing that is saved, loaded, imported or exported, so how a
//! file is shaped is decided here and nowhere else: the sections and what each holds, with numbers written back exactly as they were
//! read (`21` stays `21`, `0.5` stays `0.5`), flags staying `0`/`1`, key order kept, and whatever this version does not know
//! (a section, a key, a column field) kept as it is.
//!
//! The settings screens are generated from a schema (`data/l.json`) that names settings by string key, so the sections are keyed
//! maps, with a typed value in each and a keyed get/set on top. A file that does not fit these types is repaired before it gets here:
//! see `import`.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Section names of the settings JSON document.
pub(super) const OPTIONS_SECTION: &str = "q";
pub(super) const COLORS_SECTION: &str = "Color";
pub(super) const RANGES_SECTION: &str = "Range";
pub(super) const ALIASES_SECTION: &str = "Alias";
pub(super) const ORDER_SECTION: &str = "Order";
pub(super) const COLUMNS_SECTION: &str = "ColData";

/// A JSON number that is written back as it was read. It also converts like the original's JavaScript did: a number as text reads
/// as that number (`import` turns those into numbers when a file is loaded).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JsonNumber(serde_json::Number);

impl JsonNumber {
    /// Prints as an integer when it has no fractional part (a slider moved to 30 is saved as `30`, not `30.0`).
    pub fn from_f64(value: f64) -> Self {
        let number = if value.fract() == 0.0 && value.abs() < 9e15 {
            serde_json::Number::from(value as i64)
        } else {
            serde_json::Number::from_f64(value).unwrap_or_else(|| 0.into())
        };
        JsonNumber(number)
    }

    pub fn as_f64(&self) -> f64 {
        self.0.as_f64().unwrap_or(0.0)
    }

    /// JavaScript truthiness of a number: everything but 0.
    pub fn is_truthy(&self) -> bool {
        self.as_f64() != 0.0
    }

    pub fn to_value(&self) -> Value {
        Value::Number(self.0.clone())
    }
}

impl std::fmt::Display for JsonNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A colour as the hex digits the file has, without `#` (`03A9F4`), in the case it was written. Anything typed into a colour box is
/// stored as typed, so it is not restricted to six digits here; what draws it decides what to do with a value that is not a colour.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hex(String);

impl Hex {
    pub fn new(digits: &str) -> Self {
        Hex(digits.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An option (`q`): a switch (`0` / `1`), a choice (a number, or its key as text), or text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionValue {
    Number(JsonNumber),
    Text(String),
    Flag(bool),
}

impl OptionValue {
    /// `None` for what an option cannot be (null, a list, an object).
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Number(number) => Some(OptionValue::Number(JsonNumber(number.clone()))),
            Value::String(text) => Some(OptionValue::Text(text.clone())),
            Value::Bool(flag) => Some(OptionValue::Flag(*flag)),
            _ => None,
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            OptionValue::Number(number) => number.to_value(),
            OptionValue::Text(text) => Value::String(text.clone()),
            OptionValue::Flag(flag) => Value::Bool(*flag),
        }
    }

    pub fn switch(on: bool) -> Self {
        OptionValue::Number(JsonNumber::from_f64(f64::from(i32::from(on))))
    }

    /// JavaScript truthiness: `0`, `""`, `"0"`, `"false"` and `false` are falsy.
    pub fn is_truthy(&self) -> bool {
        match self {
            OptionValue::Flag(flag) => *flag,
            OptionValue::Number(number) => number.is_truthy(),
            OptionValue::Text(text) => !(text.is_empty() || text == "0" || text == "false"),
        }
    }

    /// The number it stands for (`0` when it is not numeric).
    pub fn as_number(&self) -> f64 {
        match self {
            OptionValue::Number(number) => number.as_f64(),
            OptionValue::Text(text) => text.trim().parse().unwrap_or(0.0),
            OptionValue::Flag(flag) => f64::from(i32::from(*flag)),
        }
    }

    /// The text it stands for; a switch written as `true` / `false` has none.
    pub fn as_text(&self) -> String {
        match self {
            OptionValue::Text(text) => text.clone(),
            OptionValue::Number(number) => number.to_string(),
            OptionValue::Flag(_) => String::new(),
        }
    }
}

/// `25` or `"100%"`, exactly as the file has it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Width {
    Number(JsonNumber),
    Text(String),
}

impl Width {
    /// `0` for a width in percent: only the number is a size in tenths of a rem.
    pub fn as_f64(&self) -> f64 {
        match self {
            Width::Number(number) => number.as_f64(),
            Width::Text(text) => text.trim().parse().unwrap_or(0.0),
        }
    }

    fn to_text(&self) -> String {
        match self {
            Width::Number(number) => number.to_string(),
            Width::Text(text) => text.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    Center,
    Right,
}

impl Align {
    pub fn as_str(self) -> &'static str {
        match self {
            Align::Left => "left",
            Align::Center => "center",
            Align::Right => "right",
        }
    }

    fn from_text(text: &str) -> Option<Self> {
        match text {
            "left" => Some(Align::Left),
            "center" => Some(Align::Center),
            "right" => Some(Align::Right),
            _ => None,
        }
    }
}

/// One table column (`ColData`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColumnData {
    /// The title in the header.
    pub tt: String,
    /// In tenths of a rem, or `"100%"` for the column that takes the rest of the row.
    pub width: Width,
    pub padding: JsonNumber,
    #[serde(rename = "alignHeader")]
    pub align_header: Align,
    #[serde(rename = "alignBody")]
    pub align_body: Align,
    /// Fields this version does not know (the job column has `"class": "Class"`). Written between the alignments and the table
    /// switches, where the one real field of this kind sits in the original's file.
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
    /// Whether the damage table shows the column (`0` / `1`).
    #[serde(rename = "DPS")]
    pub dps: JsonNumber,
    /// Whether the healing table shows the column (`0` / `1`).
    #[serde(rename = "HPS")]
    pub hps: JsonNumber,
}

impl ColumnData {
    /// A field by the name the settings screens use for it, as text (what a number or text field reads as); empty when there is none.
    pub fn text(&self, field: &str) -> String {
        match field {
            "tt" => self.tt.clone(),
            "width" => self.width.to_text(),
            "padding" => self.padding.to_string(),
            "alignHeader" => self.align_header.as_str().to_string(),
            "alignBody" => self.align_body.as_str().to_string(),
            "DPS" => self.dps.to_string(),
            "HPS" => self.hps.to_string(),
            other => match self.extra.get(other) {
                Some(Value::String(text)) => text.clone(),
                Some(Value::Number(number)) => number.to_string(),
                _ => String::new(),
            },
        }
    }

    /// A field as a number (`0` when it is not one).
    pub fn number(&self, field: &str) -> f64 {
        match field {
            "width" => self.width.as_f64(),
            "padding" => self.padding.as_f64(),
            "DPS" => self.dps.as_f64(),
            "HPS" => self.hps.as_f64(),
            other => self.text(other).trim().parse().unwrap_or(0.0),
        }
    }

    /// A field with JavaScript truthiness (`"DPS"` and `"HPS"` are the table switches).
    pub fn is_on(&self, field: &str) -> bool {
        match field {
            "DPS" => self.dps.is_truthy(),
            "HPS" => self.hps.is_truthy(),
            other => self.extra.get(other).is_some_and(super::json_coercion::is_truthy),
        }
    }

    /// Sets a field by the name the settings screens use. A value that does not fit the field is ignored.
    pub fn set(&mut self, field: &str, value: &Value) {
        let number = || match value {
            Value::Number(number) => Some(JsonNumber(number.clone())),
            Value::String(text) => text.trim().parse().ok().map(JsonNumber::from_f64),
            _ => None,
        };
        match field {
            "tt" => self.tt = value.as_str().map_or_else(|| value.to_string(), str::to_string),
            "width" => match value {
                Value::String(text) if text.trim().parse::<f64>().is_err() => self.width = Width::Text(text.clone()),
                _ => {
                    if let Some(number) = number() {
                        self.width = Width::Number(number)
                    }
                }
            },
            "padding" => self.padding = number().unwrap_or_else(|| self.padding.clone()),
            "DPS" => self.dps = number().unwrap_or_else(|| self.dps.clone()),
            "HPS" => self.hps = number().unwrap_or_else(|| self.hps.clone()),
            "alignHeader" => self.align_header = value.as_str().and_then(Align::from_text).unwrap_or(self.align_header),
            "alignBody" => self.align_body = value.as_str().and_then(Align::from_text).unwrap_or(self.align_body),
            other => {
                self.extra.insert(other.to_string(), value.clone());
            }
        }
    }
}

/// Which columns each table shows, in order (`Order`): `{"DPS": [...], "HPS": [...]}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColumnOrder(pub IndexMap<String, Vec<String>>);

/// The whole settings file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingsFile {
    /// Switches, choices and text.
    #[serde(rename = "q")]
    pub options: IndexMap<String, OptionValue>,
    #[serde(rename = "Color")]
    pub colors: IndexMap<String, Hex>,
    /// Sizes (in tenths of a rem) and opacities (in percent).
    #[serde(rename = "Range")]
    pub ranges: IndexMap<String, JsonNumber>,
    /// Action name -> the short name shown instead.
    #[serde(rename = "Alias")]
    pub aliases: IndexMap<String, String>,
    #[serde(rename = "Order")]
    pub order: ColumnOrder,
    #[serde(rename = "ColData")]
    pub columns: IndexMap<String, ColumnData>,
    /// Sections this version does not know, from a file written by another one; they are saved back as they were.
    #[serde(flatten)]
    pub unknown: IndexMap<String, Value>,
}

impl SettingsFile {
    pub fn get_option(&self, key: &str) -> Option<&OptionValue> {
        self.options.get(key)
    }

    /// Changes an option, or adds it at the end.
    pub fn set_option(&mut self, key: &str, value: OptionValue) {
        self.options.insert(key.to_string(), value);
    }

    /// The columns a table shows (`"DPS"` or `"HPS"`), in order: `Order`, without a column whose switch for the table is off or that has no
    /// definition. Everything that draws a table's header or body takes its columns from here, so they cannot disagree about which exist.
    /// (They agree with `Order` alone for everything the settings screens write, which keep the two in step.)
    pub fn enabled_columns(&self, table_label: &str) -> Vec<&str> {
        let in_order = self.order.0.get(table_label).into_iter().flatten();
        in_order.filter(|name| self.columns.get(*name).is_some_and(|definition| definition.is_on(table_label))).map(String::as_str).collect()
    }

    pub fn get_range(&self, key: &str) -> Option<f64> {
        self.ranges.get(key).map(JsonNumber::as_f64)
    }

    pub fn set_range(&mut self, key: &str, value: f64) {
        self.ranges.insert(key.to_string(), JsonNumber::from_f64(value));
    }

    pub fn get_color(&self, key: &str) -> Option<&str> {
        self.colors.get(key).map(Hex::as_str)
    }

    pub fn set_color(&mut self, key: &str, digits: &str) {
        self.colors.insert(key.to_string(), Hex::new(digits));
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/settings_file.rs"]
mod tests;
