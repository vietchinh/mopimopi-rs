//! Saved action abbreviations with a remove button each.

use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::{settings_row, settings_row_with_icon_cell};
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;

/// Saved abbreviations (long action name -> short name) with a remove button each.
pub(in crate::presentation::ui::settings_screens) fn abbreviation_list(page_context: &RowContext) -> Element {
    let settings_context = page_context.settings_context;
    let schema = &translations().ui_schema["abbset"];
    let heading = translate(&schema["in_abbOld"]["m"]).split('(').next().unwrap_or("").to_string();
    let heading_sub = translate(&schema["in_abbNew"]["m"]);
    let rows = page_context.settings.action_abbreviations().into_iter().map(|(action, short)| {
        let target = action.clone();
        let remove = rsx! {
            span { class: "removeBtn", onclick: move |_| settings_context.edit_settings(|settings| settings.remove_action_abbreviation(&target)),
                MaterialIcon { name: "remove_circle_outline" }
            }
        };
        rsx! { li { key: "{action}", class: "li_box", {settings_row_with_icon_cell("arrow_right", &action, Some(("ac", &short)), remove, "gIcon removeBtn")} } }
    });
    rsx! {
        ul { class: "remove group shadow",
            li { class: "li_box", {settings_row("arrow_right", &heading, Some(("ac", &heading_sub)), None)} }
            {rows}
        }
    }
}

