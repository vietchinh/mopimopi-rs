//! Moving between screens and settings pages.

use dioxus::prelude::*;
use super::app_actions::AppActions;
use super::contexts::{Screen, SettingsLocation};
use super::standby_mode::restart_standby_timer;
use super::toast_notifications::dismiss_toast_message;
use crate::domain::translations::translations;

/// Settings pages that show the sample tables above their rows.
pub const PAGES_WITH_LIVE_PREVIEW: [&str; 11] =
    ["Data", "font", "color", "opacity", "size", "cells", "shape", "raid", "advanced", "format", "order"];

/// Settings pages whose rows are split into tabs.
pub const PAGES_WITH_TABS: [&str; 10] = ["Data", "color", "opacity", "size", "advanced", "cells", "shape", "raid", "format", "order"];

pub fn open_settings_screen(actions: AppActions) {
    let mut screen = actions.screen.current_screen;
    screen.set(Screen::Settings);
    let mut location = actions.settings_screen.settings_location;
    location.set(SettingsLocation::top_level());
    let mut dropdown = actions.dropdown.open_dropdown;
    dropdown.set(None);
    let mut preview = actions.settings_screen.settings_preview_enabled;
    preview.set(true);
    let mut raid_preview = actions.settings_screen.settings_preview_raid_mode;
    raid_preview.set(false);
    dismiss_toast_message(actions);
}

pub fn open_settings_page(actions: AppActions, page: &str) {
    let mut location = actions.settings_screen.settings_location;
    location.set(SettingsLocation { page: page.to_string(), tab: None });
    let mut preview = actions.settings_screen.settings_preview_enabled;
    preview.set(true);
    let mut raid_preview = actions.settings_screen.settings_preview_raid_mode;
    raid_preview.set(page == "raid");
}

pub fn select_settings_tab(actions: AppActions, tab: &str) {
    let mut location = actions.settings_screen.settings_location;
    let page = location.peek().page.clone();
    location.set(SettingsLocation { page, tab: Some(tab.to_string()) });
}

/// Leaves the settings and shows the newest data again.
pub fn return_to_main_screen(actions: AppActions) {
    let mut screen = actions.screen.current_screen;
    screen.set(Screen::Main);
    let mut dropdown = actions.dropdown.open_dropdown;
    dropdown.set(None);
    let mut preview = actions.settings_screen.settings_preview_enabled;
    preview.set(false);
    // let newest = actions.latest_combat_data.peek().clone();
    // if let Some(newest) = newest.filter(|_| *actions.tables.has_received_data.peek()) {
    //     let mut displayed = actions.tables.displayed_combat_data;
    //     displayed.set(Some(newest));
    // }
    restart_standby_timer(actions);
}

/// Back arrow of the settings screen: to the parent page, or out of the settings from the top page.
pub fn go_back_in_settings(actions: AppActions) {
    let page = actions.settings_screen.settings_location.peek().page.clone();
    if page == "Settings" {
        return_to_main_screen(actions);
        return;
    }
    let parent_page = translations().ui_schema["back"][page.as_str()].as_str().unwrap_or("Settings").to_string();
    let mut location = actions.settings_screen.settings_location;
    location.set(SettingsLocation { page: parent_page, tab: None });
    let mut preview = actions.settings_screen.settings_preview_enabled;
    preview.set(true);
    let mut raid_preview = actions.settings_screen.settings_preview_raid_mode;
    raid_preview.set(false);
}
