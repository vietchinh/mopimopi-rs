//! Helpers that reproduce a few JavaScript number behaviours the original overlay relied on.
//!
//! ACT sends numbers as formatted text ("6,703.94", "12%"), and the original JavaScript code
//! converted them with `parseFloat`, `toFixed` and `Number.toString`. Keeping the same results
//! keeps the displayed values identical to the original overlay.

/// Rounds to two decimal places, like `parseFloat(x.toFixed(2))`. Non-finite values pass through.
pub fn round_to_two_decimals(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    format!("{value:.2}").parse().unwrap_or(0.0)
}

/// JavaScript `parseFloat` for text that only contains digits and dots: reads the longest
/// valid numeric prefix and returns 0 when there is none.
pub fn parse_leading_float(text: &str) -> f64 {
    let mut prefix_end = 0;
    let mut seen_decimal_point = false;
    for (index, character) in text.char_indices() {
        if character.is_ascii_digit() {
            prefix_end = index + 1;
        } else if character == '.' && !seen_decimal_point {
            seen_decimal_point = true;
            prefix_end = index + 1;
        } else {
            break;
        }
    }
    text[..prefix_end].parse::<f64>().unwrap_or(0.0)
}

/// `Number.prototype.toString()` for the values used in keys: integers print without decimals.
pub fn number_to_javascript_string(value: f64) -> String {
    const LARGEST_EXACT_INTEGER: f64 = 1e15;
    if value.fract() == 0.0 && value.abs() < LARGEST_EXACT_INTEGER {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// `Number.prototype.toFixed(decimals)`; with zero decimals ties round away from zero.
pub fn to_fixed(value: f64, decimals: usize) -> String {
    if decimals == 0 {
        format!("{:.0}", value.round())
    } else {
        format!("{value:.decimals$}")
    }
}

#[cfg(test)]
#[path = "../../tests/unit/common/javascript_compat.rs"]
mod tests;
