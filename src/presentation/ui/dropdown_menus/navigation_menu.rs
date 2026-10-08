//! The ⋮ menu. Its entries come from `ui_schema.NAV.main.dr` / `ui_schema.NAV.settings.dr`.

use super::menu_item::menu_item;
use crate::application::app_state::{open_settings_screen, return_to_main_screen, toggle_fullscreen_mode, AppActions, Screen, ScreenContext, SettingsContext, SettingsScreenContext};
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use dioxus::prelude::*;

/// One entry of the schema's menu definition.
struct MenuEntry {
    id: String,
    /// `dr_checkbox` (a switch) or `dr_link` (an action).
    kind: String,
    label_html: String,
}

pub(super) fn navigation_menu_items(actions: AppActions, screen_context: ScreenContext, settings: SettingsContext, settings_screen: SettingsScreenContext) -> Element {
    let screen = *screen_context.current_screen.read();
    let menu_key = if screen == Screen::Settings { "settings" } else { "main" };
    let entries: Vec<MenuEntry> = translations().ui_schema["NAV"][menu_key]["dr"]
        .as_object()
        .map(|definitions| {
            definitions
                .iter()
                .map(|(id, definition)| MenuEntry {
                    id: id.clone(),
                    kind: definition["e"].as_str().unwrap_or("").to_string(),
                    label_html: translate(&definition["tt"]),
                })
                .collect()
        })
        .unwrap_or_default();

    let lines = entries.into_iter().map(|entry| {
        let entry_id = entry.id.clone();
        if entry.kind == "dr_checkbox" {
            let is_on = is_menu_switch_on(settings, settings_screen, &entry.id);
            menu_item(&entry.id, &entry.label_html, Some(is_on), move |()| toggle_menu_switch(settings, settings_screen, &entry_id))
        } else {
            menu_item(&entry.id, &entry.label_html, None, move |()| run_menu_action(actions, &entry_id))
        }
    });
    rsx! {
        {lines}
    }
}

fn is_menu_switch_on(settings: SettingsContext, settings_screen: SettingsScreenContext, entry_id: &str) -> bool {
    if entry_id == "preview" {
        *settings_screen.settings_preview_enabled.read()
    } else {
        settings.settings.read().option_enabled(entry_id)
    }
}

fn toggle_menu_switch(settings: SettingsContext, settings_screen: SettingsScreenContext, entry_id: &str) {
    if entry_id == "preview" {
        let mut preview_enabled = settings_screen.settings_preview_enabled;
        let was_enabled = *preview_enabled.peek();
        preview_enabled.set(!was_enabled);
    } else {
        settings.edit_settings(|values| {
            let was_enabled = values.option_enabled(entry_id);
            values.set_option_enabled(entry_id, !was_enabled);
        });
    }
}

fn run_menu_action(actions: AppActions, entry_id: &str) {
    match entry_id {
        "fullscreen" => toggle_fullscreen_mode(),
        "settings" => open_settings_screen(actions),
        "home" => return_to_main_screen(actions),
        _ => {}
    }
}
