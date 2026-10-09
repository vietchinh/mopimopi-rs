//! What happens when the user submits text or picks a file on a settings page.

use super::base64_encoding::base64_encode;
use super::row_context::TypedTexts;
use super::text_box::clear_typed_text;
use crate::application::app_state::{show_toast_message, AppActions, SettingsContext};
use dioxus::prelude::*;
use serde_json::json;

/// Enter / send button on a text box. The box id says what entry edits:
/// * `in_apply`          – paste of a shared "Custom UI Data" code
/// * `headerText_<col>`  – custom title of a table column
/// * `in_<setting>`      – any other setting (fonts)
pub(super) fn submit_text(settings_context: SettingsContext, actions: AppActions, inputs: TypedTexts, box_id: &str, text: &str) {
    let text = text.trim().to_string();
    if text.is_empty() {
        show_toast_message(actions, "noInput", 500, 3000);
        return;
    }
    if box_id == "in_apply" {
        let mut ok = false;
        settings_context.edit_settings(|settings| ok = settings.import_shareable_code(&text).is_ok());
        show_toast_message(actions, if ok { "submit" } else { "notData" }, 500, 3000);
    } else if let Some(col) = box_id.strip_prefix("headerText_") {
        settings_context.edit_settings(|settings| settings.set_column_field(col, "tt", json!(text)));
        show_toast_message(actions, "submit", 500, 3000);
    } else if let Some(setting) = box_id.strip_prefix("in_") {
        settings_context.edit_settings(|settings| settings.set_option(setting, json!(text)));
        show_toast_message(actions, "submit", 500, 3000);
    }
    clear_typed_text(inputs, box_id);
}

/// "Add to list" on the abbreviation page: both boxes must be filled.
pub(super) fn add_abbreviation(settings_context: SettingsContext, actions: AppActions, inputs: TypedTexts) {
    let get = |id: &str| inputs.peek().get(id).cloned().unwrap_or_default().trim().to_string();
    let (action, short) = (get("in_abbOld"), get("in_abbNew"));
    if action.is_empty() || short.is_empty() {
        show_toast_message(actions, "noInput", 500, 3000);
        return;
    }
    settings_context.edit_settings(|settings| settings.set_action_abbreviation(&action, &short));
    clear_typed_text(inputs, "in_abbOld");
    clear_typed_text(inputs, "in_abbNew");
    show_toast_message(actions, "ok", 500, 3000);
}

/// Store a picked image as the overlay background (as a data: URL, so no server is needed).
pub(super) fn set_background(settings_context: SettingsContext, actions: AppActions, mime: &str, bytes: &[u8]) {
    let url = format!("data:{mime};base64,{}", base64_encode(bytes));
    settings_context.edit_settings(|settings| {
        settings.set_option("overlayBgImg", json!(url));
        settings.set_option_enabled("overlayBg", true);
    });
    show_toast_message(actions, "submit", 0, 3000);
}
