//! Top bar of the settings screen.

use crate::application::app_state::{go_back_in_settings, AppActions, Dropdown, DropdownContext, SettingsScreenContext};
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;
use serde_json::Value;

pub(super) fn page_title(page: &str) -> String {
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
    translate(title_entry)
}

#[component]
pub fn SettingsNavigationBar() -> Element {
    let actions = use_context::<AppActions>();
    let dropdown = use_context::<DropdownContext>();
    let settings_screen = use_context::<SettingsScreenContext>();
    let title = page_title(&settings_screen.settings_location.read().page);
    rsx! {
        nav { "name": "settings", class: "shadow",
            div { class: "left top btn_wrap",
                div { "name": "Back", class: "btn flex", onclick: move |_| go_back_in_settings(actions),
                    MaterialIcon { name: "arrow_back" }
                }
            }
            table { class: "nav_title", tbody { tr { td { "{title}" } } } }
            div { class: "right top btn_wrap",
                div {
                    "name": "More",
                    class: "btn flex",
                    onclick: move |_| {
                        let mut open_dropdown = dropdown.open_dropdown;
                        open_dropdown.set(Some(Dropdown::Navigation));
                    },
                    MaterialIcon { name: "more_vert" }
                }
            }
        }
    }
}
