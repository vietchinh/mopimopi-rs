//! Top bar of the settings screen.

use crate::application::app_state::*;
use crate::domain::translations::{translate, translations};
use dioxus::prelude::*;
use serde_json::Value;

pub(super) fn page_title(page: &str, lang: &str) -> String {
    let schema = &translations().ui_schema;
    let title_entry = match page {
        "Settings" => &schema["NAV"]["settings"]["tt"],
        "Data" | "Design" | "Overlay" | "Tool" => &schema["Settings"][page]["tt"],
        "font" | "color" | "opacity" | "size" | "cells" | "shape" | "raid" | "advanced" => &schema["Design"][page]["tt"],
        "format" | "order" => &schema["Data"]["tab_general"]["inner"][page]["tt"],
        "abbset" => &schema["Data"]["tab_mhh"]["inner"]["abbset"]["tt"],
        "custom" => &schema["Tool"]["custom"]["tt"],
        _ => &Value::Null,
    };
    translate(title_entry, lang)
}

#[component]
pub fn SettingsNavigationBar() -> Element {
    let context = use_context::<AppContext>();
    let title = page_title(&context.settings_location.read().page, &context.language_code());
    rsx! {
        nav { "name": "settings", class: "shadow",
            div { class: "left top btn_wrap",
                div { "name": "Back", class: "btn flex", onclick: move |_| go_back_in_settings(context),
                    i { class: "material-icons", "arrow_back" }
                }
            }
            table { class: "nav_title", tbody { tr { td { "{title}" } } } }
            div { class: "right top btn_wrap",
                div {
                    "name": "More",
                    class: "btn flex",
                    onclick: move |_| {
                        let mut open_dropdown = context.open_dropdown;
                        open_dropdown.set(Some(Dropdown::Navigation));
                    },
                    i { class: "material-icons", "more_vert" }
                }
            }
        }
    }
}
