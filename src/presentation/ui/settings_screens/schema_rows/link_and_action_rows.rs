//! Rows that open pages or run actions.

use super::{icon, note, title};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::{settings_row, settings_row_with_icon_cell};
use crate::application::app_state::{open_settings_page, AppContext, reset_settings_to_defaults, back_up_settings, restore_settings_from_backup, Dropdown};
use crate::application::i18n::{message, translate};
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;
use serde_json::Value;
use crate::presentation::ui::settings_screens::page_content::SchemaEntry;

/// Opens another settings page (the entry id is the page name).
pub(super) fn link_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let (context, id) = (page_context.context, entry.id.clone());
    let arrow = rsx! { MaterialIcon { name: "arrow_forward" } };
    rsx! {
        li { key: "{entry.id}", id: "{entry.id}", onclick: move |_| open_settings_page(context, &id),
            {settings_row_with_icon_cell(&icon(entry), &title(entry), None, arrow, "gIcon")}
        }
    }
}

/// Reset / refresh / backup / restore, job filters, and plain info lines.
pub(super) fn action_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let (context, id) = (page_context.context, entry.id.clone());
    let filters = entry.definition["dr"].as_object().cloned(); // present on the "Job Filter" rows
    let second_line = if let Some(f) = &filters {
        // names of the enabled filters, e.g. "Tank, Healer"
        let names: Vec<String> = f.iter().filter(|(k, _)| page_context.settings.option_enabled(k)).map(|(_, v)| translate(&v["tt"])).collect();
        names.join(&message("comma"))
    } else if entry.id == "backup" {
        page_context.settings.option_text("backupDate")
    } else {
        note(entry)
    };
    rsx! {
        li { key: "{entry.id}", id: "{entry.id}", onclick: move |_| run_action(context, &id, filters.as_ref()),
            {settings_row(&icon(entry), &title(entry), Some(("ac", &second_line)), None)}
        }
    }
}

pub(super) fn run_action(context: AppContext, id: &str, filters: Option<&serde_json::Map<String, Value>>) {
    match id {
        "init" => reset_settings_to_defaults(context),
        "refresh" => {
            if let Some(w) = web_sys::window() {
                let _ = w.location().reload();
            }
        }
        "backup" => back_up_settings(context),
        "restore" => restore_settings_from_backup(context),
        "DPSfilter" | "HPSfilter" => {
            let items = filters
                .map(|f| f.iter().map(|(k, v)| (k.clone(), translate(&v["tt"]))).collect())
                .unwrap_or_default();
            let mut open_dropdown = context.open_dropdown;
            open_dropdown.set(Some(Dropdown::ToggleOptions { options: items }));
        }
        _ => {}
    }
}
