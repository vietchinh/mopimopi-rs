//! Custom serde deserializers for the loosely typed scalar values ACT sends.
//!
//! Use them with `#[serde(default, deserialize_with = "...")]` on struct fields.

use crate::common::javascript_compat::{number_to_javascript_string, parse_leading_float, round_to_two_decimals};
use serde::de::{self, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::fmt;

/// Reads a value of any JSON type without building an intermediate value: each visitor below
/// receives the scalar straight from the parser (no buffering, no allocation for numbers).
/// Arrays and objects are skipped and count as "no value".
macro_rules! ignore_containers {
    () => {
        fn visit_seq<A: SeqAccess<'de>>(self, mut items: A) -> Result<Self::Value, A::Error> {
            while items.next_element::<IgnoredAny>()?.is_some() {}
            Ok(Self::empty_value())
        }

        fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
            while entries.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
            Ok(Self::empty_value())
        }
    };
}

struct NumberVisitor;

impl NumberVisitor {
    fn empty_value() -> f64 {
        0.0
    }
}

impl<'de> Visitor<'de> for NumberVisitor {
    type Value = f64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a number or ACT's formatted number text")
    }

    fn visit_bool<E: de::Error>(self, _flag: bool) -> Result<f64, E> {
        Ok(0.0)
    }
    fn visit_i64<E: de::Error>(self, number: i64) -> Result<f64, E> {
        Ok(number as f64)
    }
    fn visit_u64<E: de::Error>(self, number: u64) -> Result<f64, E> {
        Ok(number as f64)
    }
    fn visit_f64<E: de::Error>(self, number: f64) -> Result<f64, E> {
        Ok(round_to_two_decimals(number))
    }
    fn visit_str<E: de::Error>(self, text: &str) -> Result<f64, E> {
        Ok(parse_formatted_number(text).unwrap_or(0.0))
    }
    fn visit_none<E: de::Error>(self) -> Result<f64, E> {
        Ok(0.0)
    }
    fn visit_unit<E: de::Error>(self) -> Result<f64, E> {
        Ok(0.0)
    }
    ignore_containers!();
}

struct TextVisitor;

impl TextVisitor {
    fn empty_value() -> String {
        String::new()
    }
}

impl<'de> Visitor<'de> for TextVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("text, or a number / boolean that is written as text")
    }

    fn visit_bool<E: de::Error>(self, flag: bool) -> Result<String, E> {
        Ok(flag.to_string())
    }
    fn visit_i64<E: de::Error>(self, number: i64) -> Result<String, E> {
        Ok(number.to_string())
    }
    fn visit_u64<E: de::Error>(self, number: u64) -> Result<String, E> {
        Ok(number.to_string())
    }
    fn visit_f64<E: de::Error>(self, number: f64) -> Result<String, E> {
        Ok(number_to_javascript_string(number))
    }
    fn visit_str<E: de::Error>(self, text: &str) -> Result<String, E> {
        Ok(text.to_owned())
    }
    fn visit_string<E: de::Error>(self, text: String) -> Result<String, E> {
        Ok(text)
    }
    fn visit_none<E: de::Error>(self) -> Result<String, E> {
        Ok(String::new())
    }
    fn visit_unit<E: de::Error>(self) -> Result<String, E> {
        Ok(String::new())
    }
    ignore_containers!();
}

struct BoolVisitor;

impl BoolVisitor {
    fn empty_value() -> bool {
        false
    }
}

impl<'de> Visitor<'de> for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean or the text \"true\"")
    }

    fn visit_bool<E: de::Error>(self, flag: bool) -> Result<bool, E> {
        Ok(flag)
    }
    fn visit_i64<E: de::Error>(self, _number: i64) -> Result<bool, E> {
        Ok(false)
    }
    fn visit_u64<E: de::Error>(self, _number: u64) -> Result<bool, E> {
        Ok(false)
    }
    fn visit_f64<E: de::Error>(self, _number: f64) -> Result<bool, E> {
        Ok(false)
    }
    fn visit_str<E: de::Error>(self, text: &str) -> Result<bool, E> {
        Ok(text == "true")
    }
    fn visit_none<E: de::Error>(self) -> Result<bool, E> {
        Ok(false)
    }
    fn visit_unit<E: de::Error>(self) -> Result<bool, E> {
        Ok(false)
    }
    ignore_containers!();
}

/// Text that contains something other than digits, `.`, `,` and `%` names something
/// ("Broil-8,765", "00:35") rather than being a formatted number ("6,703.94", "12%").
pub fn describes_a_name_not_a_number(text: &str) -> bool {
    text.chars().any(|character| !(character.is_ascii_digit() || matches!(character, '.' | ',' | '%')))
}

/// Parses ACT's formatted number text. Returns `None` when the text is not a number
/// ("Broil-8,765"); the placeholders "---" and "--" mean zero.
pub fn parse_formatted_number(text: &str) -> Option<f64> {
    if text == "---" || text == "--" {
        return Some(0.0);
    }
    if describes_a_name_not_a_number(text) {
        return None;
    }
    let digits_only: String = text.chars().filter(|character| !matches!(character, ',' | '%')).collect();
    Some(round_to_two_decimals(parse_leading_float(&digits_only)))
}

/// A number field: accepts JSON numbers and formatted text; anything else becomes 0.
pub fn lenient_number<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    deserializer.deserialize_any(NumberVisitor)
}

/// A text field: accepts JSON strings and (as their printed form) numbers and booleans.
pub fn lenient_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    deserializer.deserialize_any(TextVisitor)
}

/// A boolean field that may arrive as `true` or as the text `"true"`.
pub fn lenient_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    deserializer.deserialize_any(BoolVisitor)
}

#[cfg(test)]
#[path = "../../../tests/unit/models/act_data/lenient_values.rs"]
mod tests;
