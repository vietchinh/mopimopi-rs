//! Settings pages that are built from the current settings rather than from `l.json`:
//! column order, column width / margin / alignment / header text, and the abbreviation list.
//!
//! * `column_titles`          – the columns to list and how to title them
//! * `column_order_page`      – reorder columns
//! * `column_size_page`       – width and margin sliders
//! * `column_alignment_page`  – header and body alignment
//! * `header_text_page`       – custom header titles
//! * `abbreviation_list`      – saved abbreviations

mod abbreviation_list;
mod column_alignment_page;
mod column_order_page;
mod column_size_page;
mod column_titles;
mod header_text_page;

pub(in crate::presentation::ui::settings_screens) use abbreviation_list::abbreviation_list;
pub(in crate::presentation::ui::settings_screens) use column_order_page::column_order_page;

use super::row_context::RowContext;
use dioxus::prelude::*;

/// The width / margin / alignment / header-text tabs of the "cells" page.
pub(super) fn column_settings_page(page_context: &RowContext, tab: &str) -> Element {
    match tab {
        "tab_width" => column_size_page::column_size_page(page_context, "width", "sizeWidth", 200.0),
        "tab_padding" => column_size_page::column_size_page(page_context, "padding", "sizePadding", 10.0),
        "tab_align" => column_alignment_page::column_alignment_page(page_context),
        _ => header_text_page::header_text_page(page_context),
    }
}
