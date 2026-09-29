//! Unit tests for `application::i18n`. They are compiled as a child module of that file but live here, outside `src/`.
//!
//! The important one: the migration from the combined JSON files (kept in `tools/i18n-source/`) to Fluent files must
//! not have changed a single character of a single text, in any language.

use super::*;
use dioxus_i18n::fluent::{FluentBundle, FluentResource};
use serde_json::json;

const LOCALES: [(&str, &str, &str); 6] = [
    ("KR", "ko-KR", include_str!("../../../src/locales/ko-KR.ftl")),
    ("JP", "ja-JP", include_str!("../../../src/locales/ja-JP.ftl")),
    ("EN", "en-US", include_str!("../../../src/locales/en-US.ftl")),
    ("FR", "fr-FR", include_str!("../../../src/locales/fr-FR.ftl")),
    ("DE", "de-DE", include_str!("../../../src/locales/de-DE.ftl")),
    ("CN", "zh-CN", include_str!("../../../src/locales/zh-CN.ftl")),
];
const ORIGINAL_LAYOUT: &str = include_str!("../../../tools/i18n-source/l.json");
const ORIGINAL_DICTIONARY: &str = include_str!("../../../tools/i18n-source/d.json");

type Bundle = FluentBundle<FluentResource>;

/// Built like `dioxus-i18n` builds its bundles (default settings), so the text is what the app shows.
fn bundle(tag: &str, source: &str) -> Bundle {
    let mut bundle = FluentBundle::new(vec![tag.parse().expect("a language tag")]);
    bundle.add_resource(FluentResource::try_new(source.to_string()).expect("valid Fluent")).expect("no message defined twice");
    bundle
}

fn text_of(bundle: &Bundle, id: &str) -> Option<String> {
    let pattern = bundle.get_message(id)?.value()?;
    let mut errors = vec![];
    let text = bundle.format_pattern(pattern, None, &mut errors).to_string();
    assert!(errors.is_empty(), "{id}: {errors:?}");
    Some(text)
}

/// The same rule the split tool used: strings keyed by two-letter codes, at least one of them a language of the app.
fn is_translation(node: &Value) -> bool {
    node.as_object().is_some_and(|map| {
        !map.is_empty()
            && map.iter().all(|(key, value)| key.len() == 2 && key.chars().all(|c| c.is_ascii_uppercase()) && value.is_string())
            && map.keys().any(|key| LOCALES.iter().any(|(code, ..)| code == key))
    })
}

/// The one language code in the original data that the app has no language for: a typo for `DE` on a text that reads
/// the same in every language ("My HPS"), left out of the split.
const IGNORED_CODE: &str = "HE";

/// Walks the original and the split data side by side; every translated text must read back identically.
fn compare(original: &Value, split: &Value, bundles: &[(&str, Bundle)], path: &str, checked: &mut usize) {
    if is_translation(original) {
        let id = message_id(split).unwrap_or_else(|| panic!("{path}: expected a message id, got {split}"));
        for code in original.as_object().unwrap().keys() {
            assert!(LOCALES.iter().any(|(known, ..)| known == code) || code == IGNORED_CODE, "{path}: unexpected language code {code}");
        }
        for (code, bundle) in bundles {
            match original.get(*code).and_then(Value::as_str) {
                Some(text) => assert_eq!(text_of(bundle, id).as_deref(), Some(text), "{path} ({code}, {id})"),
                // a language the original had no text for must have none now either, so English is used as before
                None => assert!(bundle.get_message(id).is_none(), "{path} ({code}, {id}) should be missing"),
            }
        }
        *checked += 1;
        return;
    }
    match (original, split) {
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>(), "{path}: same keys");
            for (key, value) in a { compare(value, &b[key], bundles, &format!("{path}/{key}"), checked); }
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: same length");
            for (i, (x, y)) in a.iter().zip(b).enumerate() { compare(x, y, bundles, &format!("{path}/{i}"), checked); }
        }
        _ => assert_eq!(original, split, "{path}: plain data is unchanged"),
    }
}

#[test]
fn every_text_in_every_language_reads_back_exactly_as_it_was() {
    let bundles: Vec<(&str, Bundle)> = LOCALES.iter().map(|(code, tag, source)| (*code, bundle(tag, source))).collect();
    let mut checked = 0;
    let original_layout: Value = serde_json::from_str(ORIGINAL_LAYOUT).unwrap();
    let original_dictionary: Value = serde_json::from_str(ORIGINAL_DICTIONARY).unwrap();
    compare(&original_layout, &translations().ui_schema, &bundles, "l", &mut checked);
    compare(&original_dictionary, &translations().dictionary, &bundles, "d", &mut checked);
    assert_eq!(checked, 650, "the number of translated texts");
}

#[test]
fn each_language_code_has_its_own_language() {
    let tags: Vec<String> = ["KR", "JP", "EN", "FR", "DE", "CN"].iter().map(|code| language_tag(code).to_string()).collect();
    assert_eq!(tags, ["ko-KR", "ja-JP", "en-US", "fr-FR", "de-DE", "zh-CN"]);
    assert_eq!(language_tag("anything else").to_string(), "en-US");
}

#[test]
fn plain_text_is_returned_as_it_is() {
    assert_eq!(translate(&json!("plain")), "plain");
    assert_eq!(translate(&json!(3)), "");
    assert_eq!(translate(&serde_json::Value::Null), "");
}
