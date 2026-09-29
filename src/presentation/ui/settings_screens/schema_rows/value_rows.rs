//! Rows showing or editing a value: colour, slider, current value, information.

use super::{icon, note, title};
use crate::presentation::ui::settings_screens::slider_row::{slider_row, SliderSpec};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::settings_row;
use crate::presentation::ui::settings_screens::js_color::JsColorInput;
use dioxus::prelude::*;
use crate::presentation::ui::settings_screens::page_content::SchemaEntry;

/// Colour: a hex box tinted with its colour that opens the picker panel (the original's `jscolor`).
pub(super) fn color_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let hex = page_context.settings.color_hex(&entry.id);
    let controls = rsx! { JsColorInput { id: entry.id.clone(), hex } };
    rsx! { li { key: "{entry.id}", id: "{entry.id}", class: "li_box", {settings_row(&icon(entry), &title(entry), None, Some(controls))} } }
}

/// Numeric setting stored in `Range`.
pub(super) fn slider_setting_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let (context, id) = (page_context.context, entry.id.clone());
    let t = title(entry);
    let ic = icon(entry);
    slider_row(
        &SliderSpec {
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
    let second_line = if entry.id == "share" || entry.id == "apply" { note(entry) } else { page_context.settings.option_text(&entry.id) };
    rsx! { li { key: "{entry.id}", class: "li_box", "name": "{entry.id}", {settings_row("arrow_right", &title(entry), Some(("ac", &second_line)), None)} } }
}

pub(super) fn info_row(entry: &SchemaEntry) -> Element {
    rsx! { li { key: "{entry.id}", class: "li_box", {settings_row(&icon(entry), &title(entry), Some(("ex", &note(entry))), None)} } }
}
