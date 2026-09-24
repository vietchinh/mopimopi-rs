//! Settings values written as the keys of a choice list (`1`, `.`, `solid`).

use serde_json::Value;

/// The text a value has as an object key in the schema: numbers without a trailing `.0`,
/// booleans as `1` / `0`.
pub fn choice_key_of_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => match number.as_f64() {
            Some(x) if x.fract() == 0.0 => (x as i64).to_string(),
            Some(x) => x.to_string(),
            None => number.to_string(),
        },
        Value::Bool(flag) => (*flag as i32).to_string(),
        _ => String::new(),
    }
}
