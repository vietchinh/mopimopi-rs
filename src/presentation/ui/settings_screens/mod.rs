//! Settings screens. Pages are described by `src/data/l.json`; these files decide which part
//! of that JSON to show and how to turn each entry into a row.
//!
//! * `navigation_bar`  – top bar and page titles
//! * `page_content`    – which entries / special pages the current page shows
//! * `live_preview`    – sample tables above the rows
//! * `tab_bar`         – tab buttons
//! * `row_groups`      – rows grouped into boxes
//! * `schema_rows`     – one builder per schema entry type
//! * `column_pages`    – special pages built from the settings (column order, sizes, ...)
//! * `row_context`, `row_layout`, `slider_row`, `text_box`, `form_actions`, `base64_encoding` – row building blocks

mod base64_encoding;
mod column_pages;
mod form_actions;
mod live_preview;
mod navigation_bar;
mod page_content;
mod row_context;
mod row_groups;
mod row_layout;
mod schema_rows;
mod slider_row;
mod tab_bar;
mod text_box;

pub use navigation_bar::SettingsNavigationBar;

use crate::application::app_state::*;
use dioxus::prelude::*;
use live_preview::LivePreview;
use page_content::{content_for, PageContent};
use row_context::{RowContext, TypedTexts};
use row_groups::grouped_rows;
use std::collections::HashMap;
use tab_bar::tab_bar;

/// Layout: [live preview] [tab buttons] [scrollable list of rows].
#[component]
pub fn SettingsScreen() -> Element {
    let context = use_context::<AppContext>();
    let typed_texts: TypedTexts = use_signal(HashMap::new);
    let settings = context.settings.read();
    let location = context.settings_location.read().clone();
    let show_preview = PAGES_WITH_LIVE_PREVIEW.contains(&location.page.as_str()) && *context.settings_preview_enabled.read();
    let language_code = settings.language_code();

    let page_context = RowContext { context, settings: &settings, language_code: &language_code, typed_texts };
    let body = match content_for(&settings, &location) {
        PageContent::SchemaRows(entries) => grouped_rows(&page_context, &entries),
        PageContent::Abbreviations(entries) => rsx! {
            {grouped_rows(&page_context, &entries)}
            {column_pages::abbreviation_list(&page_context)}
        },
        PageContent::ColumnOrder(table_label) => column_pages::column_order_page(&page_context, &table_label),
        PageContent::ColumnSettings(tab) => column_pages::column_settings_page(&page_context, &tab),
    };

    rsx! {
        div { "name": "settings", class: "setBody",
            div { class: "previewArea", if show_preview { LivePreview {} } }
            {tab_bar(&page_context, &location)}
            div { class: "scrollArea", {body} }
        }
    }
}
