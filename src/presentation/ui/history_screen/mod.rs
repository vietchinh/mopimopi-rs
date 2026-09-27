//! Finished encounters (History screen).
//!
//! * `history_row`  – one encounter in the list

mod history_row;

use crate::application::app_state::*;
use crate::domain::translations::{translate, translations};
use crate::presentation::ui::navigation_bar::capture_screenshot;
use dioxus::prelude::*;
use history_row::history_row;

#[component]
pub fn HistoryNavigationBar() -> Element {
    let context = use_context::<AppContext>();
    let title = translate(&translations().ui_schema["NAV"]["history"]["tt"], &context.language_code());
    let is_blinking = *context.capture_flash_active.read();
    rsx! {
        nav { "name": "history",
            div { class: "left top btn_wrap",
                div { "name": "Back", class: "btn flex", onclick: move |_| close_history_screen(context),
                    i { class: "material-icons", "arrow_back" }
                }
            }
            table { class: "nav_title", tbody { tr { td { "{title}" } } } }
            div { class: "right top btn_wrap",
                div { "name": "Capture", class: "btn flex", onclick: move |_| capture_screenshot(context),
                    i { class: if is_blinking { "material-icons flash animated" } else { "material-icons" }, "camera" }
                }
            }
        }
    }
}

#[component]
pub fn HistoryScreen() -> Element {
    let context = use_context::<AppContext>();
    let entries = context.encounter_history.read().clone();
    let viewed_key = context.viewed_history_key.read().clone();
    let rows = entries.iter().map(|entry| history_row(context, entry, viewed_key.as_deref() == Some(entry.encounter_key.as_str())));

    rsx! {
        div { "name": "history", class: "histBody",
            div { id: "HISTORYHeader",
                table { class: "tableHeader",
                    tbody {
                        tr {
                            td { class: "cell_5", "View" }
                            td { class: "cell_1", "Zone" }
                            td { class: "cell_5", "Time" }
                            td { class: "cell_6", "Total DPS" }
                            td { class: "cell_6", "Total HPS" }
                            td { class: "cell_6", "My DPS" }
                            td { class: "cell_6", "My HPS" }
                            td { class: "cell_5", "Count" }
                        }
                    }
                }
            }
            div { id: "HISTORYBody", div { id: "HISTORYoldBody", {rows} } }
        }
    }
}
