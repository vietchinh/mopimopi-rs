//! One line of the history list.

use crate::application::app_state::{show_history_entry, AppContext, HistoryEntry};
use crate::domain::formatting::NumberFormat;
use dioxus::prelude::*;

pub(super) fn history_row(context: AppContext, entry: &HistoryEntry, is_shown_now: bool) -> Element {
    let number_format = NumberFormat::from_settings(&context.settings.read());
    let rate = |value: f64| number_format.format_number(value, 1.0, number_format.rate_decimals);
    let rate_or_no_data = |value: Option<f64>| value.map(&rate).unwrap_or_else(|| "No Data".into());
    let (encounter_dps, encounter_hps) = (rate(entry.encounter_dps), rate(entry.encounter_hps));
    let (local_dps, local_hps) = (rate_or_no_data(entry.local_player_dps), rate_or_no_data(entry.local_player_hps));
    let encounter_key = entry.encounter_key.clone();

    rsx! {
        div { key: "{entry.encounter_key}", class: "tableWrap", onclick: move |_| show_history_entry(context, &encounter_key),
            table { id: "{entry.encounter_key}", class: "tableBody",
                tbody {
                    tr {
                        td { class: "cell_5",
                            if is_shown_now { img { src: "images/menu/eye.svg", style: "width:1.5rem" } }
                        }
                        td { class: "cell_1", "{entry.title}", span { class: "ex", " / {entry.zone_name}" } }
                        td { class: "cell_5", "{entry.duration_text}" }
                        td { class: "cell_6", "{encounter_dps}" }
                        td { class: "cell_6", "{encounter_hps}" }
                        td { class: "cell_6 ac", "{local_dps}" }
                        td { class: "cell_6 ac", "{local_hps}" }
                        td { class: "cell_5", "{entry.encounter_number_in_zone}" }
                    }
                }
            }
        }
    }
}
