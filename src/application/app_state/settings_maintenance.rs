//! The "Management Tools" actions and fullscreen.

use dioxus::prelude::*;
use super::app_context::AppContext;
use super::toast_notifications::show_toast_message;
use crate::domain::settings::{read_local_storage, write_local_storage, Settings, BACKUP_STORAGE_KEY};
use crate::domain::translations::translations;
use serde_json::Value;

/// Asks for confirmation, then restores all settings to the defaults (keeping the language).
pub fn reset_settings_to_defaults(context: AppContext) {
    let question = translations().message("initConfirm", &context.language_code());
    let confirmed = web_sys::window().and_then(|window| window.confirm_with_message(&question).ok()).unwrap_or(false);
    if !confirmed {
        return;
    }
    let language = context.settings.peek().language_code();
    let mut fresh = Settings::defaults();
    fresh.set_option("Lang", Value::String(language));
    let mut settings = context.settings;
    settings.set(fresh);
    show_toast_message(context, "init", 0, 3000);
}

pub fn back_up_settings(context: AppContext) {
    let date = js_sys::Date::new_0().to_string().as_string().unwrap_or_default();
    context.edit_settings(|settings| settings.set_option("backupDate", Value::String(date)));
    write_local_storage(BACKUP_STORAGE_KEY, &context.settings.peek().to_json_text());
    show_toast_message(context, "backup", 500, 3000);
}

pub fn restore_settings_from_backup(context: AppContext) {
    match read_local_storage(BACKUP_STORAGE_KEY).and_then(|text| Settings::from_json_text(&text)) {
        Some(restored) => {
            let mut settings = context.settings;
            settings.set(restored);
            show_toast_message(context, "restore", 500, 3000);
        }
        None => show_toast_message(context, "noData", 0, 3000),
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
