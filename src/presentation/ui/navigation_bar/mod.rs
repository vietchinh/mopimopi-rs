//! Top bar of the main screen: encounter time, target name, DPS summary and buttons.
//! Also drawn (with `is_settings_preview`) as the sample bar on settings pages.
//!
//! * `summary_line`  – "Total DPS 0  Total HPS 0  Rank 1/1/1  MaxHit ..."
//! * `buttons`       – Capture, History, End encounter and the ⋮ button

mod buttons;
mod summary_line;

pub use buttons::capture_screenshot;

use crate::application::app_state::AppContext;
use crate::domain::translations::{translate, translations};
use buttons::NavigationButtons;
use dioxus::prelude::*;
use summary_line::summary_line;
use crate::presentation::ui::overlay_plugin_context::OverlayPluginContext;

/// Layout choice "Display Type of Combatant Data": summary below the target (2 lines) or beside it.
const TWO_LINE_LAYOUT: i32 = 2;

#[component]
pub fn NavigationBar(is_settings_preview: bool) -> Element {
    let mut overlay_plugin_context = use_context::<OverlayPluginContext>();
    let context = use_context::<AppContext>();
    let settings = context.settings.read();
    let language = settings.language_code();
    let encounter = overlay_plugin_context.get_encounter_record();
    let is_encounter_active = overlay_plugin_context.get_is_encounter_active();

    let (time_text, target_text) = match is_encounter_active {
        true => (encounter.duration_text.clone(), encounter.title.clone()),
        false => ("00:00".to_string(), translate(&translations().ui_schema["NAV"]["main"]["tt"]["target"], &language)),
    };
    let summary = summary_line(context, &settings, &language, &overlay_plugin_context.get_local_player_combatant_record(), &encounter, is_settings_preview, is_encounter_active);
    let uses_two_lines = settings.option_number("act") as i32 == TWO_LINE_LAYOUT;

    // The original has two alternative layouts (2 rows / 1 row); only one is visible.
    rsx! {
        nav { "name": "main",
            table { "name": "ACT_2line", style: if uses_two_lines { "" } else { "display:none" },
                tbody {
                    tr {
                        td { rowspan: "2", "name": "time", "{time_text}" }
                        td { "name": "target", "{target_text}" }
                    }
                    tr { td { "name": "rps", {summary.clone()} } }
                }
            }
            table { "name": "ACT_1line", style: if uses_two_lines { "display:none" } else { "" },
                tbody {
                    tr {
                        td { "name": "time", "{time_text}" }
                        td { "name": "target", "{target_text}" }
                        td { "name": "rps", {summary} }
                    }
                }
            }
            NavigationButtons { is_settings_preview }
        }
    }
}
