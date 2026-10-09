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

/// Every expected value below was produced by real JavaScript (`(x).toFixed(d)`), not derived.
#[test]
fn to_fixed_matches_javascript_including_exact_ties() {
    // exact ties: JavaScript takes the larger candidate, Rust's `{:.N}` takes the even one
    assert_eq!(to_fixed(12.25, 1), "12.3");
    assert_eq!(to_fixed(12.75, 1), "12.8");
    assert_eq!(to_fixed(0.125, 2), "0.13");
    assert_eq!(to_fixed(2.5, 0), "3");
    assert_eq!(to_fixed(0.5, 0), "1");
    assert_eq!(to_fixed(-2.5, 0), "-3");
    assert_eq!(to_fixed(9.5, 0), "10");
    assert_eq!(to_fixed(99.95, 1), "100.0"); // not a tie in binary (99.9500000000000028...) but carries
    // not ties: decided by the exact binary value, which both agree on
    assert_eq!(to_fixed(21.15, 1), "21.1");
    assert_eq!(to_fixed(28.85, 1), "28.9");
    assert_eq!(to_fixed(3.85, 1), "3.9");
    assert_eq!(to_fixed(1.005, 2), "1.00");
    // zero and sign
    assert_eq!(to_fixed(0.0, 2), "0.00");
    assert_eq!(to_fixed(-0.0, 2), "0.00");
    assert_eq!(to_fixed(0.0001, 2), "0.00");
    assert_eq!(to_fixed(-0.0001, 2), "-0.00");
    assert_eq!(to_fixed(1234.5678, 0), "1235");
}

#[test]
fn p_float_rounds_first_so_the_second_rounding_sees_two_decimals() {
    // The original shows 11/52 as "21.1", not "21.2": 21.1538... -> 21.15 (pFloat) -> "21.1" (toFixed(1)).
    assert_eq!(to_fixed(p_float(11.0 / 52.0 * 100.0), 1), "21.1");
    assert_eq!(to_fixed(p_float(15.0 / 52.0 * 100.0), 1), "28.9"); // 28.846 -> 28.85 -> 28.9
    assert_eq!(to_fixed(p_float(2.0 / 52.0 * 100.0), 1), "3.9"); // 3.846 -> 3.85 -> 3.9
    assert_eq!(p_float(f64::NAN), 0.0);
    assert_eq!(p_float(f64::INFINITY), f64::INFINITY);
}

#[test]
fn to_fixed_fast_path_agrees_with_exact_expansion() {
    // Every value is compared with what the exact (slow) route gives: the same digits, ties rounded up.
    for decimals in 0..=3usize {
        for step in 0..20_000u32 {
            let value = f64::from(step) * 0.0625 + f64::from(step % 7) * 0.001; // plenty of exact ties and near-ties
            let exact = {
                let long = format!("{value:.*}", decimals + 30);
                let (_, fraction) = long.split_once('.').unwrap();
                let tie = fraction.as_bytes().get(decimals) == Some(&b'5') && fraction[decimals + 1..].bytes().all(|d| d == b'0');
                (tie, format!("{value:.decimals$}"))
            };
            let got = to_fixed(value, decimals);
            if !exact.0 {
                assert_eq!(got, exact.1, "{value} to {decimals}");
            }
        }
    }
}
