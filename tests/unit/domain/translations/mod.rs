//! Unit tests for `translations::mod`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;
use serde_json::json;

#[test]
fn picks_language_and_falls_back_to_english() {
    let value = json!({"KR": "안녕", "EN": "Hello"});
    assert_eq!(translate(&value, "KR"), "안녕");
    assert_eq!(translate(&value, "FR"), "Hello");
    assert_eq!(translate(&json!("plain"), "KR"), "plain");
}

#[test]
fn bundled_files_contain_known_entries() {
    assert!(!translations().message("submit", "EN").is_empty());
}
