//! The settings-page layout, and which text each part of it shows.
//!
//! Two JSON files, converted from the original `lang.js` and `dic.js` by `tools/i18n-split.mjs`:
//! * `data/l.json` (`ui_schema`)   – every settings page: its rows, their types, icons and ranges
//! * `data/d.json` (`dictionary`)  – short labels: column hints, alignment names, warnings
//!
//! They hold no text. Where a row shows a translated text, the JSON holds its message id (`"i18n:l-Design-font-tt"`),
//! and the texts themselves are Project Fluent files, one per language, in `src/locales/`. Turning an id into text
//! in the language in use is `application::i18n`'s job (it needs the running app); this module stays plain data.

use serde_json::Value;
use std::sync::OnceLock;

/// What marks a JSON string as a message id rather than plain text.
pub const ID_PREFIX: &str = "i18n:";

pub struct Translations {
    pub ui_schema: Value,
    pub dictionary: Value,
}

/// The layout files, parsed once.
pub fn translations() -> &'static Translations {
    static TRANSLATIONS: OnceLock<Translations> = OnceLock::new();
    TRANSLATIONS.get_or_init(|| Translations {
        ui_schema: serde_json::from_str(include_str!("../../data/l.json")).expect("data/l.json is valid JSON"),
        dictionary: serde_json::from_str(include_str!("../../data/d.json")).expect("data/d.json is valid JSON"),
    })
}

/// The message id a translatable value stands for, or `None` for a plain string (or anything else).
pub fn message_id(value: &Value) -> Option<&str> {
    value.as_str()?.strip_prefix(ID_PREFIX)
}

impl Translations {
    /// Toast / dialog message: the translatable value at `ui_schema.msg[id].m`.
    pub fn message(&self, message_id: &str) -> &Value {
        &self.ui_schema["msg"][message_id]["m"]
    }

    /// Short title of a dictionary entry (`dictionary[key].tt`), for example "Chocobo".
    pub fn dictionary_title(&self, key: &str) -> &Value {
        &self.dictionary[key]["tt"]
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/translations/mod.rs"]
mod tests;
