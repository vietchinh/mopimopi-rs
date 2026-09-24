//! The ⋮ menu. Its entries come from `ui_schema.NAV.main.dr` / `ui_schema.NAV.settings.dr`.

use super::menu_item::menu_item;
use crate::application::app_state::*;
use crate::domain::translations::{translate, translations};
use dioxus::prelude::*;

/// One entry of the schema's menu definition.
struct MenuEntry {
    id: String,
    /// `dr_checkbox` (a switch) or `dr_link` (an action).
    kind: String,
    label_html: String,
}

pub(super) fn navigation_menu_items(context: AppContext) -> Element {
    let language = context.language_code();
    let screen = *context.current_screen.read();
    let menu_key = if screen == Screen::Settings { "settings" } else { "main" };
    let entries: Vec<MenuEntry> = translations().ui_schema["NAV"][menu_key]["dr"]
        .as_object()
        .map(|definitions| {
            definitions
                .iter()
                .map(|(id, definition)| MenuEntry {
                    id: id.clone(),
                    kind: definition["e"].as_str().unwrap_or("").to_string(),
                    label_html: translate(&definition["tt"], &language),
                })
                .collect()
        })
        .unwrap_or_default();

    let lines = entries.into_iter().map(|entry| {
        let entry_id = entry.id.clone();
        if entry.kind == "dr_checkbox" {
            let is_on = is_menu_switch_on(context, &entry.id);
            menu_item(&entry.id, &entry.label_html, Some(is_on), move |_| toggle_menu_switch(context, &entry_id))
        } else {
            menu_item(&entry.id, &entry.label_html, None, move |_| run_menu_action(context, &entry_id))
        }
    });
    rsx! {
        {lines}
    }
}

fn is_menu_switch_on(context: AppContext, entry_id: &str) -> bool {
    if entry_id == "preview" {
        *context.settings_preview_enabled.read()
    } else {
        context.settings.read().option_enabled(entry_id)
    }
}

fn toggle_menu_switch(context: AppContext, entry_id: &str) {
    if entry_id == "preview" {
        let mut preview_enabled = context.settings_preview_enabled;
        let was_enabled = *preview_enabled.peek();
        preview_enabled.set(!was_enabled);
    } else {
        context.edit_settings(|settings| {
            let was_enabled = settings.option_enabled(entry_id);
            settings.set_option_enabled(entry_id, !was_enabled);
        });
    }
}

fn run_menu_action(context: AppContext, entry_id: &str) {
    match entry_id {
        "fullscreen" => toggle_fullscreen_mode(),
        "settings" => open_settings_screen(context),
        "home" => return_to_main_screen(context),
        _ => {}
    }
}

/// Back to the start screen so another ACT address can be entered.
fn show_start_screen(context: AppContext) {
    let mut has_received_data = context.has_received_data;
    has_received_data.set(false);
    let mut dropdown = context.open_dropdown;
    dropdown.set(None);
}
