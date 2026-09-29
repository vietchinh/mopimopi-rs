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
    to_fixed(value, 2).parse().unwrap_or(0.0)
}

/// The original's `pFloat`: `parseFloat(num.nanFix().toFixed(2))`. NaN becomes 0, infinity stays.
/// Every derived rate and percentage the original computes goes through this *before* it is
/// shown, so a value is rounded twice (first to two decimals, then to the displayed precision), and
/// that first step changes what the second one produces (21.1538 -> 21.15 -> "21.1", not "21.2").
pub fn p_float(value: f64) -> f64 {
    if value.is_nan() { 0.0 } else { round_to_two_decimals(value) }
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

/// `Number.prototype.toFixed(decimals)`.
///
/// JavaScript rounds the value's exact decimal expansion and, when it lies exactly halfway, takes
/// the larger candidate (away from zero). Rust rounds the same exact expansion but resolves an exact
/// tie to the even digit, so `12.25` -> one decimal is `12.3` in JavaScript and `12.2` with `{:.1}`.
/// Ties are common here (a percentage already rounded to `x.25` or `x.75`), so they are handled.
pub fn to_fixed(value: f64, decimals: usize) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let magnitude = value.abs();
    // A tie ends exactly one digit past the cut, so a much longer expansion shows all zeros after it.
    let long = format!("{magnitude:.*}", decimals + 30);
    let (integer, fraction) = long.split_once('.').unwrap_or((long.as_str(), ""));
    let is_tie = fraction.as_bytes().get(decimals) == Some(&b'5') && fraction[decimals + 1..].bytes().all(|digit| digit == b'0');
    if !is_tie {
        return format!("{sign}{magnitude:.decimals$}");
    }
    // Round the tie up: the kept digits plus one unit in the last place.
    let mut digits: Vec<u8> = integer.bytes().chain(fraction.bytes().take(decimals)).collect();
    let mut position = digits.len();
    loop {
        if position == 0 {
            digits.insert(0, b'1');
            break;
        }
        position -= 1;
        if digits[position] == b'9' {
            digits[position] = b'0';
        } else {
            digits[position] += 1;
            break;
        }
    }
    let text = String::from_utf8(digits).unwrap_or_default();
    let split = text.len() - decimals;
    if decimals == 0 { format!("{sign}{text}") } else { format!("{sign}{}.{}", &text[..split], &text[split..]) }
}

#[cfg(test)]
#[path = "../../tests/unit/common/javascript_compat.rs"]
mod tests;
