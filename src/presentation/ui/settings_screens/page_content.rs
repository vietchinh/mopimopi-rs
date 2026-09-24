//! Which content a settings page shows, taken from the schema in `data/l.json`.

use crate::application::app_state::{SettingsLocation, PAGES_WITH_TABS};
use crate::domain::settings::Settings;
use crate::domain::translations::translations;
use serde_json::Value;

/// One entry of a schema page (`id` is its key in l.json, `obj` its definition).
#[derive(Clone, PartialEq)]
pub(super) struct SchemaEntry {
    /// Key of the entry in `l.json` (a setting key, a page name, ...).
    pub(super) id: String,
    /// The entry's definition (type `e`, texts `tt` / `m`, limits, ...).
    pub(super) definition: Value,
    /// "DPS" / "HPS" on the column pages, otherwise `None`.
    pub(super) table_label: Option<String>,
}

/// What the scroll area of the current page shows.
pub(super) enum PageContent {
    /// Ordinary rows from the schema.
    SchemaRows(Vec<SchemaEntry>),
    /// Column order list for "DPS" or "HPS".
    ColumnOrder(String),
    /// Width / margin / align / header-text pages ("tab_width", ...).
    ColumnSettings(String),
    /// Abbreviation page: a form plus the list of saved abbreviations.
    Abbreviations(Vec<SchemaEntry>),
}

pub(super) fn entries_of(obj: &Value, flag: Option<&str>) -> Vec<SchemaEntry> {
    obj.as_object()
        .map(|m| {
            m.iter()
                .filter(|(_, v)| v.is_object())
                .map(|(k, v)| SchemaEntry { id: k.clone(), definition: v.clone(), table_label: flag.map(String::from) })
                .collect()
        })
        .unwrap_or_default()
}

/// Tab buttons of a page (entries of type `tab_btn`), in schema order.
pub(super) fn tabs_of(page: &str) -> Vec<(String, Value)> {
    if !PAGES_WITH_TABS.contains(&page) {
        return vec![];
    }
    translations().ui_schema[page]
        .as_object()
        .map(|m| m.iter().filter(|(_, v)| v["e"] == "tab_btn").map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default()
}

/// The selected tab: the remembered one if valid, otherwise the first.
pub(super) fn current_tab(page: &str, wanted: &Option<String>) -> Option<String> {
    let tabs = tabs_of(page);
    let first = tabs.first()?.0.clone();
    Some(wanted.clone().filter(|t| tabs.iter().any(|(k, _)| k == t)).unwrap_or(first))
}

pub(super) fn content_for(settings: &Settings, location: &SettingsLocation) -> PageContent {
    let schema = &translations().ui_schema;
    let page = location.page.as_str();
    if page == "abbset" {
        return PageContent::Abbreviations(entries_of(&schema["abbset"], None));
    }
    let Some(tab) = current_tab(page, &location.tab) else {
        return PageContent::SchemaRows(entries_of(&schema[page], None)); // page without tabs
    };
    let column_flag = tab.trim_start_matches("tab_").to_string();
    match (page, tab.as_str()) {
        ("color", "tab_graph") => PageContent::SchemaRows(entries_of(&schema["Graph"][settings.option_text("palette").as_str()], None)),
        ("order", _) => PageContent::ColumnOrder(column_flag),
        ("cells", "tab_title" | "tab_width" | "tab_padding" | "tab_align") => PageContent::ColumnSettings(tab),
        ("format", _) => PageContent::SchemaRows(entries_of(&schema[page][tab.as_str()]["inner"], Some(&column_flag))),
        _ => PageContent::SchemaRows(entries_of(&schema[page][tab.as_str()]["inner"], None)),
    }
}
