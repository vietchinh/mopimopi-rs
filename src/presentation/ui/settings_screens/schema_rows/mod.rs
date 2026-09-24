//! One schema entry -> one row. `render_schema_row` picks the builder by the entry's `"e"` type.
//!
//! Entry types (see `src/data/l.json`):
//! `li_link` opens a page · `li_2line`/`li_2line_empty` action or info row · `li_radio` pick one value
//! · `li_checkbox` on/off · `li_2line_checkbox` column on/off · `li_color` · `li_slider`
//! · `li_pn` shows a current value · `li_box` text note · `li_text(_inbtn)` text box · `li_file` upload
//!
//! * `link_and_action_rows` – open a page, reset / backup / restore, job filters
//! * `choice_rows`          – pick one value, on/off switches
//! * `value_rows`           – colour, slider, current value, info text
//! * `text_rows`            – text boxes, share code, add button, file upload

mod choice_rows;
mod link_and_action_rows;
mod text_rows;
mod value_rows;

use super::page_content::SchemaEntry;
use super::row_context::RowContext;
use crate::domain::translations::translate;
use dioxus::prelude::*;

use choice_rows::*;
use link_and_action_rows::*;
use text_rows::*;
use value_rows::*;

pub(super) fn title(page_context: &RowContext, entry: &SchemaEntry) -> String {
    translate(&entry.definition["tt"], page_context.language_code)
}
pub(super) fn icon(entry: &SchemaEntry) -> String {
    entry.definition["i"].as_str().unwrap_or("").to_string()
}
/// The grey explanation text (`m`) in the current language.
pub(super) fn note(page_context: &RowContext, entry: &SchemaEntry) -> String {
    translate(&entry.definition["m"], page_context.language_code)
}

pub(super) fn render_schema_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    match entry.definition["e"].as_str().unwrap_or("") {
        "li_link" => link_row(page_context, entry),
        "li_2line" | "li_2line_empty" => action_row(page_context, entry),
        "li_radio" | "li_radio_change" => radio_row(page_context, entry),
        "li_checkbox" | "li_2line_checkbox_normal" => switch_row(page_context, entry),
        "li_2line_checkbox" => column_switch_row(page_context, entry),
        "li_color" => color_row(page_context, entry),
        "li_slider" => slider_setting_row(page_context, entry),
        "li_pn" => value_row(page_context, entry),
        "li_box" => info_row(page_context, entry),
        "li_text" if entry.id == "in_share" => share_row(page_context, entry),
        "li_text" => text_row(page_context, entry, false),
        "li_text_inbtn" => text_row(page_context, entry, true),
        "li_full_btn" => add_button_row(page_context, entry),
        "li_file" => file_row(page_context, entry),
        _ => rsx! {},
    }
}
