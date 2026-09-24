//! Translated text and the settings-page layout.
//!
//! Two JSON files, converted from the original `lang.js` and `dic.js` by `tools/extract.js`:
//! * `data/l.json` (`ui_schema`)   – every settings page: its rows, their types and their texts
//! * `data/d.json` (`dictionary`)  – short labels: column hints, alignment names, warnings
//!
//! A translated text is an object keyed by language code (`{"KR": "...", "EN": "..."}`);
//! `translate` picks the requested language and falls back to English.

use serde_json::Value;
use std::sync::OnceLock;

const FALLBACK_LANGUAGE_CODE: &str = "EN";

pub struct Translations {
    pub ui_schema: Value,
    pub dictionary: Value,
}

/// The translations, parsed once.
pub fn translations() -> &'static Translations {
    static TRANSLATIONS: OnceLock<Translations> = OnceLock::new();
    TRANSLATIONS.get_or_init(|| Translations {
        ui_schema: serde_json::from_str(include_str!("../../data/l.json")).expect("data/l.json is valid JSON"),
        dictionary: serde_json::from_str(include_str!("../../data/d.json")).expect("data/d.json is valid JSON"),
    })
}

/// Text of a translatable value: a plain string, or an object keyed by language code.
pub fn translate(value: &Value, language_code: &str) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Object(by_language) => by_language
            .get(language_code)
            .or_else(|| by_language.get(FALLBACK_LANGUAGE_CODE))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    }
}

impl Translations {
    /// Toast / dialog message (`ui_schema.msg[id].m`).
    pub fn message(&self, message_id: &str, language_code: &str) -> String {
        translate(&self.ui_schema["msg"][message_id]["m"], language_code)
    }

    /// Short title of a dictionary entry (`dictionary[key].tt`), for example "Chocobo".
    pub fn dictionary_title(&self, key: &str, language_code: &str) -> String {
        translate(&self.dictionary[key]["tt"], language_code)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/translations/mod.rs"]
mod tests;
