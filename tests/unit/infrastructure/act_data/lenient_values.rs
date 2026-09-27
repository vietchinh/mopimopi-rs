//! Unit tests for `infrastructure::act::data::lenient_values`. Compiled as a child module of that
//! file (so they can use its private items) but kept here, outside `src/`.

use super::*;

fn number(json: &str) -> f64 {
    #[derive(serde::Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "lenient_number")] f64);
    serde_json::from_str::<Wrapper>(json).unwrap().0
}

fn rate(json: &str) -> f64 {
    #[derive(serde::Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "lenient_rate")] f64);
    serde_json::from_str::<Wrapper>(json).unwrap().0
}

fn text(json: &str) -> String {
    #[derive(serde::Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "lenient_text")] String);
    serde_json::from_str::<Wrapper>(json).unwrap().0
}

fn action_name(json: &str) -> String {
    #[derive(serde::Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "lenient_action_name")] String);
    serde_json::from_str::<Wrapper>(json).unwrap().0
}

fn boolean(json: &str) -> bool {
    #[derive(serde::Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "lenient_bool")] bool);
    serde_json::from_str::<Wrapper>(json).unwrap().0
}

#[test]
fn numbers_accept_json_numbers_formatted_text_and_placeholders() {
    assert_eq!(number("342515"), 342515.0);
    assert_eq!(number(r#""6,703.94""#), 6703.94);
    assert_eq!(number(r#""12%""#), 12.0);
    assert_eq!(number(r#""---""#), 0.0);
    assert_eq!(number(r#""--""#), 0.0);
    assert_eq!(number("null"), 0.0);
    assert_eq!(number("true"), 0.0);
}

#[test]
fn a_name_is_not_a_number() {
    assert_eq!(number(r#""Broil-8,765""#), 0.0);
    assert_eq!(parse_formatted_number("Broil-8,765"), None);
}

#[test]
fn rates_keep_infinity_but_plain_numbers_do_not() {
    assert_eq!(rate(r#""∞""#), f64::INFINITY);
    assert_eq!(rate(r#""Infinity""#), f64::INFINITY);
    assert_eq!(number(r#""∞""#), 0.0, "a plain number field has no use for infinity");
}

#[test]
fn text_accepts_strings_and_prints_other_scalars() {
    assert_eq!(text(r#""Shirogane""#), "Shirogane");
    assert_eq!(text("true"), "true");
    assert_eq!(text("42"), "42");
    assert_eq!(text("null"), "");
}

#[test]
fn action_names_drop_a_trailing_amount_but_keep_hyphenated_names() {
    assert_eq!(action_name(r#""Shieldsplitter-1819""#), "Shieldsplitter");
    assert_eq!(action_name(r#""Broil-8,765""#), "Broil");
    assert_eq!(action_name(r#""Divine Veil (*)""#), "Divine Veil (*)");
    assert_eq!(action_name(r#""item_ffc95""#), "item_ffc95");
    assert_eq!(action_name(r#""""#), "");
}

#[test]
fn bool_accepts_json_booleans_and_the_text_true() {
    assert!(boolean("true"));
    assert!(boolean(r#""true""#));
    assert!(!boolean("false"));
    assert!(!boolean(r#""false""#));
    assert!(!boolean("null"));
}
