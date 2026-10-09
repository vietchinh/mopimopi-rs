//! Unit tests for `translations::mod`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;
use serde_json::json;

#[test]
fn a_message_id_is_recognised_and_plain_text_is_not() {
    assert_eq!(message_id(&json!("i18n:l-Design-font-tt")), Some("l-Design-font-tt"));
    assert_eq!(message_id(&json!("just text")), None);
    assert_eq!(message_id(&json!(3)), None);
    assert_eq!(message_id(&serde_json::Value::Null), None);
}

#[test]
fn bundled_files_refer_to_texts_by_id_and_hold_no_language_objects() {
    // A translated text in these files is an id. An object keyed by language code whose values are text (not ids) would be
    // text that was never split out. (The `Lang` setting's list of options is keyed by language code too, but holds ids.)
    fn no_language_objects(node: &Value) {
        match node {
            Value::Object(map) => {
                let unsplit = map.iter().any(|(key, value)| ["KR", "JP", "EN", "FR", "DE", "CN"].contains(&key.as_str()) && value.as_str().is_some_and(|text| !text.starts_with(ID_PREFIX)));
                assert!(!unsplit, "still holds language texts: {map:?}");
                map.values().for_each(no_language_objects);
            }
            Value::Array(items) => items.iter().for_each(no_language_objects),
            _ => {}
        }
    }
    no_language_objects(&translations().ui_schema);
    no_language_objects(&translations().dictionary);
    assert!(message_id(translations().message("submit")).is_some());
    assert!(message_id(translations().dictionary_title("LMB")).is_some());
}
