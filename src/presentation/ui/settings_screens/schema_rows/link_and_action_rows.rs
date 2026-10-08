//! Rows that open pages or run actions.

use super::{icon, note, title};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::{settings_row, settings_row_with_icon_cell};
use crate::application::app_state::{AppActions, Dropdown, DropdownContext, back_up_settings, open_settings_page, reset_settings_to_defaults, restore_settings_from_backup};
use crate::application::i18n::{message, translate};
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;
use serde_json::Value;
use crate::presentation::ui::settings_screens::page_content::SchemaEntry;

/// Opens another settings page (the entry id is the page name).
pub(super) fn link_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let (actions, id) = (page_context.actions, entry.id.clone());
    let arrow = rsx! { MaterialIcon { name: "arrow_forward" } };
    rsx! {
        li { key: "{entry.id}", id: "{entry.id}", onclick: move |_| open_settings_page(actions, &id),
            {settings_row_with_icon_cell(&icon(entry), &title(entry), None, arrow, "gIcon")}
        }
    }
}

/// Reset / refresh / backup / restore, job filters, and plain info lines.
pub(super) fn action_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let (actions, dropdown, id) = (page_context.actions, page_context.dropdown, entry.id.clone());
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
        li { key: "{entry.id}", id: "{entry.id}", onclick: move |_| run_action(actions, dropdown, &id, filters.as_ref()),
            {settings_row(&icon(entry), &title(entry), Some(("ac", &second_line)), None)}
        }
    }
}

pub(super) fn run_action(actions: AppActions, dropdown: DropdownContext, id: &str, filters: Option<&serde_json::Map<String, Value>>) {
    match id {
        "init" => reset_settings_to_defaults(actions),
        "refresh" => {
            if let Some(w) = web_sys::window() {
                let _ = w.location().reload();
            }
        }
        "backup" => back_up_settings(actions),
        "restore" => restore_settings_from_backup(actions),
        "DPSfilter" | "HPSfilter" => {
            let items = filters
                .map(|f| f.iter().map(|(k, v)| (k.clone(), translate(&v["tt"]))).collect())
                .unwrap_or_default();
            let mut open_dropdown = dropdown.open_dropdown;
            open_dropdown.set(Some(Dropdown::ToggleOptions { options: items }));
        }
        _ => {}
    }
}
