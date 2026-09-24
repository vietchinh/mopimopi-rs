//! The main screen before any data arrived (start screen), and the switch to the tables.
//!
//! * `language_links`  – "Please select your language: 한국어 | English | ..."
//! * `connect_box`     – ACT address box for a hosted copy (not in the original)

mod language_links;

use super::combat_tables::CombatTables;
use crate::application::app_state::AppContext;
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
        div { "name": "main", class: "mainBody", style: if is_standby_hidden { "display:none" } else { "" },
            CombatTables { is_settings_preview: false }
        }
    }
}

#[component]
fn StartScreen() -> Element {
    let context = use_context::<AppContext>();
    let is_korean = context.language_code() == "KR";
    // Original behaviour: Korean text for Korean users, English for everyone else.
    let notice_text = |key: &str| {
        let entry = &translations().ui_schema["Notice"][key];
        entry[if is_korean { "KR" } else { "EN" }].as_str().unwrap_or("").to_string()
    };
    // The static HTML before the language line comes from the translation file.
    let introduction = notice_text("strong").split("Please select").next().unwrap_or("").to_string();
    let (tip, update_notes) = (notice_text("tip"), notice_text("update"));

    rsx! {
        div { "name": "notice", class: "noticeBody",
            div { id: "strong",
                span { {markup_view(&introduction)} }
                LanguageLinks {}
            }
            div { id: "tip", {markup_view(&tip)} }
            div { id: "update", {markup_view(&update_notes)} }
        }
    }
}
