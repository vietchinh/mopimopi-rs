//! One line of the history list.

use crate::application::app_state::{show_history_entry, AppContext, HistoryEntry};
use crate::domain::formatting::NumberFormat;
use dioxus::prelude::*;

const CELL_BORDER: &str = " not-last:[border-right:var(--chrome-cell-border)]";

pub(super) fn history_row(context: AppContext, entry: &HistoryEntry, is_shown_now: bool) -> Element {
    let number_format = NumberFormat::from_settings(&context.settings.read());
    let rate = |value: f64| number_format.format_number(value, 1.0, number_format.rate_decimals);
    let rate_or_no_data = |value: Option<f64>| value.map_or_else(|| "No Data".into(), &rate);
    let (encounter_dps, encounter_hps) = (rate(entry.encounter_dps()), rate(entry.encounter_hps()));
    let (local_dps, local_hps) = (rate_or_no_data(entry.local_player_dps()), rate_or_no_data(entry.local_player_hps()));
    let (title, zone_name, duration_text) = (entry.title(), entry.zone_name(), entry.duration_text());
    let encounter_key = entry.encounter_key.clone();

    rsx! {
        // History has no "own row" concept (see HistoryScreen's comment): always table-text-other.
        // Everything here (row height, corners, the .ex size, own/other text) is read from the
        // variables `HistoryScreen` sets once on #HISTORYBody, via standard_table's `body_style`.
        div { key: "{entry.encounter_key}", class: "tableWrap table-text-other chrome-row", onclick: move |_| show_history_entry(context, &encounter_key),
            table { id: "{entry.encounter_key}", class: "tableBody",
                tbody {
                    tr {
                        td { class: "cell_5{CELL_BORDER}",
                            if is_shown_now { img { src: "images/menu/eye.svg", style: "width:1.5rem" } }
                        }
                        td { class: "cell_1{CELL_BORDER}", "{title}", span { class: "ex chrome-ex", " / {zone_name}" } }
                        td { class: "cell_5{CELL_BORDER}", "{duration_text}" }
                        td { class: "cell_6{CELL_BORDER}", "{encounter_dps}" }
                        td { class: "cell_6{CELL_BORDER}", "{encounter_hps}" }
                        td { class: "cell_6 ac{CELL_BORDER}", "{local_dps}" }
                        td { class: "cell_6 ac{CELL_BORDER}", "{local_hps}" }
                        td { class: "cell_5{CELL_BORDER}", "{entry.encounter_number_in_zone}" }
                    }
                }
            }
            // The row's actual visible background: `.tableBody` itself is transparent
            // (`position:absolute`, drawn on top), so without this the row shows straight through
            // to the page's own background. The original creates this same element for every
            // history row too (`process.js`, `historyAddRow`): `barBg.className = "barBg"`.
            div { class: "barBg chrome-bar-bg corner-body" }
        }
    }
}
