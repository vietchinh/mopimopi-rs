//! The "Management Tools" actions and fullscreen.

use dioxus::prelude::*;
use super::app_actions::AppActions;
use super::toast_notifications::show_toast_message;
use crate::domain::settings::{Settings, SettingKey};
use crate::infrastructure::browser_storage::{load_backup, save_backup};
use crate::application::i18n::message;
use serde_json::Value;

/// Asks for confirmation, then restores all settings to the defaults (keeping the language).
pub fn reset_settings_to_defaults(actions: AppActions) {
    let question = message("initConfirm");
    let confirmed = web_sys::window().and_then(|window| window.confirm_with_message(&question).ok()).unwrap_or(false);
    if !confirmed {
        return;
    }
    let language = actions.settings.settings.peek().language_code();
    let mut fresh = Settings::defaults();
    fresh.set(SettingKey::Language, Value::String(language));
    let mut settings = actions.settings.settings;
    settings.set(fresh);
    show_toast_message(actions, "init", 0, 3000);
}

pub fn back_up_settings(actions: AppActions) {
    let date = js_sys::Date::new_0().to_string().as_string().unwrap_or_default();
    actions.settings.edit_settings(|settings| settings.set(SettingKey::BackupDate, Value::String(date)));
    save_backup(&actions.settings.settings.peek());
    show_toast_message(actions, "backup", 500, 3000);
}

pub fn restore_settings_from_backup(actions: AppActions) {
    match load_backup() {
        Some(restored) => {
            let mut settings = actions.settings.settings;
            settings.set(restored);
            show_toast_message(actions, "restore", 500, 3000);
        }
        None => show_toast_message(actions, "noData", 0, 3000),
    }
}

pub fn toggle_fullscreen_mode() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else { return };
    if document.fullscreen_element().is_some() {
        document.exit_fullscreen();
    } else if let Some(root_element) = document.document_element() {
        let _ = root_element.request_fullscreen();
    }
}
