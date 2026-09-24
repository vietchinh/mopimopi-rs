//! Unit tests for `javascript_compat`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn rounds_like_to_fixed_two() {
    assert_eq!(round_to_two_decimals(6703.944), 6703.94);
    assert_eq!(round_to_two_decimals(f64::INFINITY), f64::INFINITY);
}

#[test]
fn parses_leading_float_prefix() {
    assert_eq!(parse_leading_float("12.5.3"), 12.5);
    assert_eq!(parse_leading_float(""), 0.0);
}

#[test]
fn prints_integers_without_decimals() {
    assert_eq!(number_to_javascript_string(35.0), "35");
    assert_eq!(number_to_javascript_string(35.5), "35.5");
}
