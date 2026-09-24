//! Unit tests for `act_data::lenient_values`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[derive(serde::Deserialize)]
struct Sample {
    #[serde(default, deserialize_with = "lenient_number")]
    number: f64,
    #[serde(default, deserialize_with = "lenient_text")]
    text: String,
    #[serde(default, deserialize_with = "lenient_bool")]
    flag: bool,
}

fn sample(json: &str) -> Sample {
    serde_json::from_str(json).expect("sample JSON parses")
}

#[test]
fn numbers_accept_formatted_text_and_placeholders() {
    assert_eq!(sample(r#"{"number":"6,703.94"}"#).number, 6703.94);
    assert_eq!(sample(r#"{"number":"12%"}"#).number, 12.0);
    assert_eq!(sample(r#"{"number":"---"}"#).number, 0.0);
    assert_eq!(sample(r#"{"number":""}"#).number, 0.0);
    assert_eq!(sample(r#"{"number":"Broil-8,765"}"#).number, 0.0);
    assert_eq!(sample(r#"{"number":42}"#).number, 42.0);
    assert_eq!(sample(r#"{"number":null}"#).number, 0.0);
    assert_eq!(sample(r#"{}"#).number, 0.0);
}

#[test]
fn text_accepts_strings_numbers_and_missing() {
    assert_eq!(sample(r#"{"text":"00:35"}"#).text, "00:35");
    assert_eq!(sample(r#"{"text":35}"#).text, "35");
    assert_eq!(sample(r#"{"text":null}"#).text, "");
}

#[test]
fn booleans_accept_text() {
    assert!(sample(r#"{"flag":"true"}"#).flag);
    assert!(sample(r#"{"flag":true}"#).flag);
    assert!(!sample(r#"{"flag":"false"}"#).flag);
}

#[test]
fn names_are_told_apart_from_numbers() {
    assert!(describes_a_name_not_a_number("Broil-8,765"));
    assert!(describes_a_name_not_a_number("00:35"));
    assert!(!describes_a_name_not_a_number("6,703.94"));
}
