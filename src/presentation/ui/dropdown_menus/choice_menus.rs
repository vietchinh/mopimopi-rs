//! Menus for picking values: one of several, several on/off options, or a column alignment.

use super::menu_item::menu_item;
use crate::application::app_state::SettingsContext;
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use crate::presentation::ui::shared::option_choice::choice_key_of_value;
use dioxus::prelude::*;
use serde_json::{json, Value};

/// Pick one value of a setting. `choices` maps each stored value to its translated label.
pub(super) fn option_choice_items(settings_context: SettingsContext, setting_key: &str, choices: &Value) -> Element {
    let current_value = choice_key_of_value(&settings_context.settings.read().option_value(setting_key));
    let lines = choices.as_object().into_iter().flatten().map(|(value, label)| {
        let (setting_key, value) = (setting_key.to_string(), value.clone());
        let is_current = current_value == value;
        menu_item(&value.clone(), &translate(label), Some(is_current), move |()| {
            settings_context.edit_settings(|settings| settings.set_option_from_text(&setting_key, &value));
        })
    });
    rsx! { {lines} }
}

/// Several independent on/off options (the job filters).
pub(super) fn option_toggle_items(settings_context: SettingsContext, options: Vec<(String, String)>) -> Element {
    let lines = options.into_iter().map(|(setting_key, label)| {
        let is_on = settings_context.settings.read().option_enabled(&setting_key);
        menu_item(&setting_key.clone(), &label, Some(is_on), move |()| {
            settings_context.edit_settings(|settings| {
                let was_on = settings.option_enabled(&setting_key);
                settings.set_option_enabled(&setting_key, !was_on);
            });
        })
    });
    rsx! { {lines} }
}

/// Left / center / right for a column's header or body text.
pub(super) fn column_alignment_items(settings_context: SettingsContext, column: &str, field: &str) -> Element {
    let current_alignment = settings_context.settings.read().column_text(column, field);
    let alignments = translations().dictionary["direction"].as_object().into_iter().flatten();
    let lines = alignments.map(|(alignment, label)| {
        let (column, field, alignment) = (column.to_string(), field.to_string(), alignment.clone());
        let is_current = current_alignment == alignment;
        menu_item(&alignment.clone(), &translate(label), Some(is_current), move |()| {
            settings_context.edit_settings(|settings| settings.set_column_field(&column, &field, json!(alignment)));
        })
    });
    rsx! { {lines} }
}
