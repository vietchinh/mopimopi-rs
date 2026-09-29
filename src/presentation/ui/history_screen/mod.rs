//! Finished encounters (History screen).
//!
//! * `history_row`  – one encounter in the list

mod history_row;

use crate::application::app_state::{AppContext, close_history_screen};
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use crate::presentation::ui::navigation_bar::{capture_screenshot, nav_bar_style};
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;
use history_row::history_row;

#[component]
pub fn HistoryNavigationBar() -> Element {
    let context = use_context::<AppContext>();
    let title = translate(&translations().ui_schema["NAV"]["history"]["tt"]);
    let is_blinking = *context.capture_flash_active.read();
    let nav = nav_bar_style(&context.settings.read());
    rsx! {
        nav { "name": "history", class: "nav-bar", style: "{nav}",
            div { class: "left top btn_wrap",
                div { "name": "Back", class: "btn flex", onclick: move |_| close_history_screen(context),
                    span { class: "nav-icon", MaterialIcon { name: "arrow_back" } }
                }
            }
            table { class: "nav_title", tbody { tr { td { "{title}" } } } }
            div { class: "right top btn_wrap",
                div { "name": "Capture", class: "btn flex", onclick: move |_| capture_screenshot(context),
                    span { class: "nav-icon", class: if is_blinking { "flash animated" }, MaterialIcon { name: "camera" } }
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
    let settings = context.settings.read();
    let values = crate::presentation::ui::shared::style_values::StyleValues::new(&settings);
    let header = crate::presentation::ui::combat_tables::standard_table::header_style(&values);
    let body = crate::presentation::ui::combat_tables::standard_table::body_style(&values);
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
                class: "{class} chrome-header-cell first:corner-header-left last:corner-header-right not-last:[border-right:var(--chrome-header-cell-border)]",
                "{label}"
            }
        }
    });

    let entries = context.encounter_history.read().clone();
    let rows = entries
        .iter()
        .map(|entry| history_row(context, entry, viewed_key.as_deref() == Some(entry.encounter_key.as_str())));

    rsx! {
        div { "name": "history", class: "histBody",
            div { id: "HISTORYHeader", style: "{header}",
                table { class: "tableHeader chrome-header-table",
                    tbody {
                        tr {
                            {header_cells}
                        }
                    }
                }
            }
            div { id: "HISTORYBody", style: "{body}", div { id: "HISTORYoldBody", {rows} } }
        }
    }
}
