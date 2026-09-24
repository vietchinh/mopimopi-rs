//! Pick-one rows and on/off switches.

use super::{icon, note, title};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::settings_row;
use crate::application::app_state::*;
use crate::domain::translations::translate;
use crate::presentation::ui::shared::option_choice::choice_key_of_value;
use crate::presentation::ui::shared::switch_and_icon::SwitchToggle;
use dioxus::prelude::*;
use crate::presentation::ui::settings_screens::page_content::SchemaEntry;

/// "Pick one of several": shows the current choice, opens a radio dropdown on click.
pub(super) fn radio_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let context = page_context.context;
    let current = choice_key_of_value(page_context.settings.option_value(&entry.id));
    let label = translate(&entry.definition["m"][current.as_str()], page_context.language_code);
    // `li_radio_change` puts the label inside a sentence: "... ★ ..." -> "... label ..."
    let second_line = if entry.definition["e"] == "li_radio_change" { translate(&entry.definition["msg"], page_context.language_code).replace('★', &label) } else { label };
    let (id, options) = (entry.id.clone(), entry.definition["m"].clone());
    rsx! {
        li { key: "{entry.id}", id: "{entry.id}", class: "radio",
            onclick: move |_| {
                let mut open_dropdown = context.open_dropdown;
                open_dropdown.set(Some(Dropdown::ChooseOption { setting_key: id.clone(), choices: options.clone() }));
            },
            {settings_row(&icon(entry), &title(page_context, entry), Some(("ac", &second_line)), None)}
        }
    }
}

/// On/off setting.
pub(super) fn switch_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let (context, id) = (page_context.context, entry.id.clone());
    let on = page_context.settings.option_enabled(&entry.id);
    let explanation = note(page_context, entry);
    let sub = if entry.definition["e"] == "li_checkbox" { None } else { Some(("ac", explanation.as_str())) };
    rsx! {
        li { key: "{entry.id}", id: "{entry.id}",
            onclick: move |_| context.edit_settings(|settings| {
                let was_enabled = settings.option_enabled(&id);
                settings.set_option_enabled(&id, !was_enabled);
            }),
            {settings_row(&icon(entry), &title(page_context, entry), sub, Some(rsx! { SwitchToggle { is_on: on } }))}
        }
    }
}

/// On/off for a table column (in the DPS or HPS table).
pub(super) fn column_switch_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let context = page_context.context;
    let flag = entry.table_label.clone().unwrap_or_else(|| "DPS".into());
    let on = page_context.settings.column_number(&entry.id, &flag) != 0.0;
    let (column, table) = (entry.id.clone(), flag.clone());
    let name = entry.definition["tt"].as_str().unwrap_or(&entry.id).to_string(); // column names are plain strings
    rsx! {
        li { key: "{flag}-{entry.id}", id: "{flag}-{entry.id}",
            onclick: move |_| context.edit_settings(|settings| settings.set_column_enabled(&column, &table, !on)),
            {settings_row("arrow_right", &name, Some(("ex", &note(page_context, entry))), Some(rsx! { SwitchToggle { is_on: on } }))}
        }
    }
}
