//! The main screen before any data arrived (start screen), and the switch to the tables.
//!
//! * `language_links`    – "Please select your language: 한국어 | English | ..."
//! * `connection_panel`  – connection status and the sample-data button

mod language_links;

use super::combat_tables::CombatTables;
use crate::application::app_state::AppContext;
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use crate::presentation::ui::shared::safe_markup::markup_view;
use dioxus::prelude::*;
use language_links::LanguageLinks;

/// The tables once data has arrived, the start screen before that.
#[component]
pub fn MainScreen() -> Element {
    let context = use_context::<AppContext>();
    if !*context.has_received_data.read() {
        return rsx! { StartScreen {} };
    }
    let is_standby_hidden = *context.is_standby_hidden.read();
    rsx! {
        div { "name": "main", class: "mainBody", display: if is_standby_hidden { "none" },
            CombatTables { is_settings_preview: false }
        }
    }
}

#[component]
fn StartScreen() -> Element {
    // These notices exist only in Korean and English: Korean users get Korean, everyone else falls back to English,
    // which is what the original showed.
    let notice_text = |key: &str| translate(&translations().ui_schema["Notice"][key]);
    // The static HTML before the language line comes from the translation file.
    let introduction = notice_text("strong").split("Please select").next().unwrap_or("").to_string();
    let (tip, update_notes) = (notice_text("tip"), notice_text("update"));

    rsx! {
        div { "name": "notice", class: "noticeBody",
            div { id: "strong",
                span { {markup_view(&introduction)} }
                LanguageLinks {}
                // (the original ends this block with a line break too; it has no height, but it is part of the text)
                br {}
            }
            div { id: "tip", {markup_view(&tip)} }
            div { id: "update", {markup_view(&update_notes)} }
        }
    }
}
