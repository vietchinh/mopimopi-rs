//! "Please select your language" links of the start screen.

use crate::application::app_state::{show_toast_message, AppActions, SettingsContext};
use dioxus::prelude::*;

/// (language code, the language's own name).
const LANGUAGES: [(&str, &str); 6] =
    [("KR", "한국어"), ("EN", "English"), ("JP", "日本語"), ("CN", "中國語"), ("DE", "Deutsch"), ("FR", "French")];

#[component]
pub(super) fn LanguageLinks() -> Element {
    let settings = use_context::<SettingsContext>();
    let actions = use_context::<AppActions>();
    rsx! {
        "Please select "
        b { "your language" }
        " : "
        for (index , (language_code , language_name)) in LANGUAGES.into_iter().enumerate() {
            Fragment { key: "{language_code}",
            if index > 0 { " ❘ " }
            a {
                id: "{language_code}",
                href: "#",
                onclick: move |event| {
                    event.prevent_default();
                    settings.edit_settings(|settings| settings.set_option("Lang", serde_json::json!(language_code)));
                    show_toast_message(actions, "submit", 0, 3000);
                },
                "{language_name}"
            }
            }
        }
    }
}
