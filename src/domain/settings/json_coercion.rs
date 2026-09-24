//! JavaScript-style coercions the original code relied on when reading settings values.
//! Settings written by the original overlay mix numbers, booleans and strings freely.

use serde_json::{json, Value};

/// JavaScript truthiness: `0`, `""`, `"0"`, `"false"`, `null` and `false` are falsy.
pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().map(|x| x != 0.0).unwrap_or(false),
        Value::String(text) => !(text.is_empty() || text == "0" || text == "false"),
        Value::Null => false,
        _ => true,
    }
}

/// Number value of a JSON value (`0` when it is not numeric).
pub(super) fn as_number(value: &Value) -> f64 {
    match value {
        Value::Number(number) => number.as_f64().unwrap_or(0.0),
        Value::String(text) => text.trim().parse().unwrap_or(0.0),
        Value::Bool(flag) => *flag as i32 as f64,
        _ => 0.0,
    }
}

/// JSON number that prints as an integer when it has no fractional part.
pub(super) fn number_to_json(number: f64) -> Value {
    if number.fract() == 0.0 && number.abs() < 9e15 {
        json!(number as i64)
    } else {
        json!(number)
    }
}
