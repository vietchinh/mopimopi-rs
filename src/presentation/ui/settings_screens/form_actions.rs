//! What happens when the user submits text or picks a file on a settings page.

use super::base64_encoding::base64_encode;
use super::row_context::TypedTexts;
use super::text_box::clear_typed_text;
use crate::application::app_state::*;
use dioxus::prelude::*;
use serde_json::json;

/// Enter / send button on a text box. The box id says what entry edits:
/// * `in_apply`          – paste of a shared "Custom UI Data" code
/// * `headerText_<col>`  – custom title of a table column
/// * `in_<setting>`      – any other setting (fonts)
pub(super) fn submit_text(context: AppContext, inputs: TypedTexts, box_id: &str, text: String) {
    let text = text.trim().to_string();
    if text.is_empty() {
        show_toast_message(context, "noInput", 500, 3000);
        return;
    }
    if box_id == "in_apply" {
        let mut ok = false;
        context.edit_settings(|settings| ok = settings.import_shareable_code(&text).is_ok());
        show_toast_message(context, if ok { "submit" } else { "notData" }, 500, 3000);
    } else if let Some(col) = box_id.strip_prefix("headerText_") {
        context.edit_settings(|settings| settings.set_column_field(col, "tt", json!(text)));
        show_toast_message(context, "submit", 500, 3000);
    } else if let Some(setting) = box_id.strip_prefix("in_") {
        context.edit_settings(|settings| settings.set_option(setting, json!(text)));
        show_toast_message(context, "submit", 500, 3000);
    }
    clear_typed_text(inputs, box_id);
}

/// "Add to list" on the abbreviation page: both boxes must be filled.
pub(super) fn add_abbreviation(context: AppContext, inputs: TypedTexts) {
    let get = |id: &str| inputs.peek().get(id).cloned().unwrap_or_default().trim().to_string();
    let (action, short) = (get("in_abbOld"), get("in_abbNew"));
    if action.is_empty() || short.is_empty() {
        show_toast_message(context, "noInput", 500, 3000);
        return;
    }
    context.edit_settings(|settings| settings.set_action_abbreviation(&action, &short));
    clear_typed_text(inputs, "in_abbOld");
    clear_typed_text(inputs, "in_abbNew");
    show_toast_message(context, "ok", 500, 3000);
}

/// Store a picked image as the overlay background (as a data: URL, so no server is needed).
pub(super) fn set_background(context: AppContext, mime: &str, bytes: &[u8]) {
    let url = format!("data:{mime};base64,{}", base64_encode(bytes));
    context.edit_settings(|settings| {
        settings.set_option("overlayBgImg", json!(url));
        settings.set_option_enabled("overlayBg", true);
    });
    show_toast_message(context, "submit", 0, 3000);
}
