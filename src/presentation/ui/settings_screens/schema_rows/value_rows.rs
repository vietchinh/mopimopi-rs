//! Rows showing or editing a value: colour, slider, current value, information.

use super::{icon, note, title};
use crate::presentation::ui::settings_screens::slider_row::{slider_row, SliderSpec};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::settings_row;
use dioxus::prelude::*;
use crate::presentation::ui::settings_screens::page_content::SchemaEntry;

/// Colour: the browser's colour picker plus a hex box.
pub(super) fn color_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let context = page_context.context;
    let hex = page_context.settings.color_hex(&entry.id);
    let (id_picker, id_hex) = (entry.id.clone(), entry.id.clone());
    let controls = rsx! {
        input {
            r#type: "color",
            value: "#{hex}",
            oninput: move |event| {
                let hex_digits = event.value().trim_start_matches('#').to_uppercase();
                if hex_digits.len() == 6 { context.edit_settings(|settings| settings.set_color_hex(&id_picker, &hex_digits)); }
            },
        }
        input {
            class: "shadow inputEff",
            style: "text-align:center;width:8rem;margin-left:.6rem",
            maxlength: "6",
            value: "{hex}",
            oninput: move |event| {
                let hex_digits = event.value().trim_start_matches('#').to_uppercase();
                if hex_digits.len() == 6 && hex_digits.chars().all(|c| c.is_ascii_hexdigit()) {
                    context.edit_settings(|settings| settings.set_color_hex(&id_hex, &hex_digits));
                }
            },
        }
    };
    rsx! { li { key: "{entry.id}", id: "{entry.id}", class: "li_box", {settings_row(&icon(entry), &title(page_context, entry), None, Some(controls))} } }
}

/// Numeric setting stored in `Range`.
pub(super) fn slider_setting_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let (context, id) = (page_context.context, entry.id.clone());
    let t = title(page_context, entry);
    let ic = icon(entry);
    slider_row(
        SliderSpec {
            id: &entry.id,
            icon: &ic,
            title: &t,
            min: entry.definition["min"].as_f64().unwrap_or(0.0),
            max: entry.definition["max"].as_f64().unwrap_or(100.0),
            value: page_context.settings.slider_value(&entry.id),
        },
        move |new_value| context.edit_settings(|settings| settings.set_slider_value(&id, new_value)),
    )
}

/// Title plus a current value (fonts) or a hint (share / apply).
pub(super) fn value_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let second_line = if entry.id == "share" || entry.id == "apply" { note(page_context, entry) } else { page_context.settings.option_text(&entry.id) };
    rsx! { li { key: "{entry.id}", class: "li_box", "name": "{entry.id}", {settings_row("arrow_right", &title(page_context, entry), Some(("ac", &second_line)), None)} } }
}

pub(super) fn info_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    rsx! { li { key: "{entry.id}", class: "li_box", {settings_row(&icon(entry), &title(page_context, entry), Some(("ex", &note(page_context, entry))), None)} } }
}
