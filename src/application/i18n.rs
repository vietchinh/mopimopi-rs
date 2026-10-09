//! Translation, on top of `dioxus-i18n` (Project Fluent).
//!
//! The texts are in `src/locales/<tag>.ftl`, one file per language, embedded in the app. The overlay keeps its own
//! language codes (`KR`, `JP`, `EN`, `FR`, `DE`, `CN`, the `Lang` setting and what people's saved settings hold);
//! `language_tag` maps them to the languages Fluent knows. A text missing in a language falls back to English, as the
//! original did.
//!
//! The settings layout (`domain::translations`) holds message ids; `translate` turns one into text.

use crate::domain::translations::{message_id, translations};
use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use dioxus_i18n::unic_langid::{langid, LanguageIdentifier};
use serde_json::Value;

/// The Fluent language for one of the overlay's language codes (English for anything unknown).
pub fn language_tag(language_code: &str) -> LanguageIdentifier {
    match language_code {
        "KR" => langid!("ko-KR"),
        "JP" => langid!("ja-JP"),
        "FR" => langid!("fr-FR"),
        "DE" => langid!("de-DE"),
        "CN" => langid!("zh-CN"),
        _ => langid!("en-US"),
    }
}

/// Sets translation up. Call once, at the top of the app, with the language code the settings hold.
pub fn use_init_translations(language_code: &str) -> I18n {
    let tag = language_tag(language_code);
    use_init_i18n(move || {
        I18nConfig::new(tag)
            .with_fallback(langid!("en-US"))
            .with_locale((langid!("ko-KR"), include_str!("../locales/ko-KR.ftl")))
            .with_locale((langid!("ja-JP"), include_str!("../locales/ja-JP.ftl")))
            .with_locale((langid!("en-US"), include_str!("../locales/en-US.ftl")))
            .with_locale((langid!("fr-FR"), include_str!("../locales/fr-FR.ftl")))
            .with_locale((langid!("de-DE"), include_str!("../locales/de-DE.ftl")))
            .with_locale((langid!("zh-CN"), include_str!("../locales/zh-CN.ftl")))
    })
}

/// Text of a translatable value from the settings layout: a message id, or a plain string (returned as it is).
/// Empty when there is no such text, in this language or in English.
pub fn translate(value: &Value) -> String {
    match message_id(value) {
        Some(id) => lookup(id),
        None => value.as_str().map(str::to_owned).unwrap_or_default(),
    }
}

/// A toast / dialog message (`ui_schema.msg[id].m`).
pub fn message(message_id: &str) -> String {
    translate(translations().message(message_id))
}

/// Short title of a dictionary entry, for example "Chocobo".
pub fn dictionary_title(key: &str) -> String {
    translate(translations().dictionary_title(key))
}

fn lookup(id: &str) -> String {
    // (not `i18n()`: it panics without the context, and text lookups also run where there is none)
    match try_consume_context::<I18n>() {
        Some(i18n) => i18n.try_translate(id).unwrap_or_default(),
        None => String::new(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/application/i18n.rs"]
mod tests;
