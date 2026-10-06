//! Finished encounters (History screen).
//!
//! * `history_row`  – one encounter in the list

mod history_row;

use crate::application::app_state::{AppContext, close_history_screen};
use crate::application::i18n::translate;
use crate::domain::formatting::NumberFormat;
use crate::domain::translations::translations;
use crate::presentation::ui::areas::SettingsView;
use crate::presentation::ui::combat_tables::cell_classes::{HEADER_CELL, HEADER_TABLE};
use crate::presentation::ui::navigation_bar::capture_screenshot;
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;
use history_row::history_row;

#[component]
pub fn HistoryNavigationBar() -> Element {
    let context = use_context::<AppContext>();
    let title = translate(&translations().ui_schema["NAV"]["history"]["tt"]);
    let is_blinking = *context.capture_flash_active.read();
    let nav = use_context::<SettingsView>().nav.read().bar_vars();
    rsx! {
        nav { "name": "history", class: "nav-bar", style: "{nav}",
            div { class: "left top btn_wrap",
                div { "name": "Back", class: "btn flex", onclick: move |_| close_history_screen(context),
                    span { class: "nav-icon text-(color:--nav-icon-color) text-(length:--nav-icon-size)", MaterialIcon { name: "arrow_back" } }
                }
            }
            table { class: "nav_title", tbody { tr { td { "{title}" } } } }
            div { class: "right top btn_wrap",
                div { "name": "Capture", class: "btn flex", onclick: move |_| capture_screenshot(context),
                    span { class: "nav-icon text-(color:--nav-icon-color) text-(length:--nav-icon-size)", class: if is_blinking { "flash animated" }, MaterialIcon { name: "camera" } }
                }
            }
        }
    }
}

#[component]
pub fn HistoryScreen() -> Element {
    let context = use_context::<AppContext>();
    let viewed_key = context.viewed_history_key.read().clone();

    // History has no concept of an "own" row (it shows encounter summaries, not per-player rows),
    // so it's always styled as "other" -- matching what the pre-Tailwind CSS selectors always
    // resolved to here anyway (History rows never carried `id="YOU"` or `.myPet`).
    let table = use_context::<SettingsView>().table.read().clone();
    let (header, body) = (table.header_vars(), table.body_vars());
    // (class, label) of each header cell. The end cells' rounded corners come from `first:`/`last:`.
    const COLUMNS: [(&str, &str); 8] = [
        ("cell_5", "View"),
        ("cell_1", "Zone"),
        ("cell_5", "Time"),
        ("cell_6", "Total DPS"),
        ("cell_6", "Total HPS"),
        ("cell_6", "My DPS"),
        ("cell_6", "My HPS"),
        ("cell_5", "Count"),
    ];
    let header_cells = COLUMNS.iter().map(|(class, label)| {
        rsx! {
            td {
                key: "{label}",
                class: "{class}{HEADER_CELL}",
                "{label}"
            }
        }
    });

    let number_format = NumberFormat::from_settings(&context.settings.read());
    let entries = context.encounter_history.read().clone();
    let rows = entries
        .iter()
        .map(|entry| history_row(context, entry, viewed_key.as_deref() == Some(entry.encounter_key.as_str()), &number_format));

    rsx! {
        div { "name": "history", class: "histBody",
            // (Not italic either whatever "Header italic" says, as for the list.)
            div { id: "HISTORYHeader", style: "{header}--text-style:normal;",
                table { class: HEADER_TABLE,
                    tbody {
                        tr {
                            {header_cells}
                        }
                    }
                }
            }
            // The list is not italic whatever "Body italic" says: that setting has only ever applied to the player rows.
            div { id: "HISTORYBody", style: "{body}--text-style:normal;", div { id: "HISTORYoldBody", {rows} } }
        }
    }
}
