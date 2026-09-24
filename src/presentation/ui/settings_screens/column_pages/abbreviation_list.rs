//! Saved action abbreviations with a remove button each.

use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::settings_row;
use crate::domain::translations::{translate, translations};
use dioxus::prelude::*;

/// Saved abbreviations (long action name -> short name) with a remove button each.
pub(in crate::presentation::ui::settings_screens) fn abbreviation_list(page_context: &RowContext) -> Element {
    let context = page_context.context;
    let schema = &translations().ui_schema["abbset"];
    let heading = translate(&schema["in_abbOld"]["m"], page_context.language_code).split('(').next().unwrap_or("").to_string();
    let heading_sub = translate(&schema["in_abbNew"]["m"], page_context.language_code);
    let rows = page_context.settings.action_abbreviations().into_iter().map(|(action, short)| {
        let target = action.clone();
        let remove = rsx! {
            span { class: "removeBtn", onclick: move |_| context.edit_settings(|settings| settings.remove_action_abbreviation(&target)),
                i { class: "material-icons", "remove_circle_outline" }
            }
        };
        rsx! { li { key: "{action}", class: "li_box", {settings_row("arrow_right", &action, Some(("ac", &short)), Some(remove))} } }
    });
    rsx! {
        ul { class: "remove group shadow",
            li { class: "li_box", {settings_row("arrow_right", &heading, Some(("ac", &heading_sub)), None)} }
            {rows}
        }
    }
}

