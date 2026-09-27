//! The DPS and HPS tables (rows with graph bars) and the compact "raid mode" grid.
//!
//! Reading order: `CombatTables` (entry) -> `standard_table` / `raid_grid` -> `player_row` -> `graph_bars`.
//!
//! * `table_environment`  – everything the drawing functions share
//! * `visible_players`    – which players a table shows
//! * `standard_table`     – header, body and one row per player
//! * `graph_bars`         – the coloured bars behind a row
//! * `raid_grid`          – the compact cards
//! * `setup_hint`         – what to check in ACT when no row named YOU arrives

mod graph_bars;
mod raid_grid;
mod standard_table;
mod table_environment;
mod visible_players;

use crate::application::app_state::AppContext;
use crate::presentation::ui::overlay_plugin_context::OverlayPluginContext;
use dioxus::prelude::*;
use table_environment::TableEnvironment;

/// "DPS" or "HPS": the prefix used in element ids, column flags and setting keys. There are only
/// ever these two tables, so a plain bool (`is_healing`) says which one is meant throughout this
/// module, rather than a dedicated enum for a distinction this local.
fn table_label(is_healing: bool) -> &'static str {
    if is_healing { "HPS" } else { "DPS" }
}

/// Shows the tables in the order and combination the user configured.
#[component]
pub fn CombatTables(is_settings_preview: bool) -> Element {
    let overlay_plugin_context = use_context::<OverlayPluginContext>();
    let context = use_context::<AppContext>();
    let settings = context.settings.read();
    let local_player_name = context.local_player_name.read().clone();
    let blurred_rows = context.blurred_player_rows.read().clone();

    // The real tables draw whatever is currently displayed (live, a history entry, or frozen
    // while settings are open); the settings preview always draws the built-in sample fight.
    let (combatants, encounter) = if is_settings_preview {
        let sample = crate::application::app_state::sample_combat_message();
        (sample.combatants.clone(), sample.encounter.clone())
    } else {
        let Some(message) = context.displayed_combat_data.read().clone() else { return rsx! {} };
        (message.combatants.clone(), message.encounter.clone())
    };

    let is_encounter_active = if is_settings_preview { true } else { overlay_plugin_context.get_is_encounter_active() };
    let environment = TableEnvironment::new(is_encounter_active, &settings, &local_player_name, &blurred_rows, is_settings_preview);

    let raid_mode = (settings.option_enabled("view24") && combatants.len() as f64 >= settings.option_number("view24_Number"))
        || (is_settings_preview && *context.settings_preview_raid_mode.read());
    let damage_table_first = settings.option_number("tableOrder") as i32 == 1;
    let tables = if damage_table_first { [false, true] } else { [true, false] };

    let sections = tables
        .into_iter()
        .filter(|&is_healing| settings.option_enabled(&format!("view{}", table_label(is_healing))))
        .map(|is_healing| {
            if raid_mode {
                raid_grid::raid_grid(&environment, &combatants, &encounter, is_healing)
            } else {
                standard_table::standard_table(&environment, &combatants, &encounter, is_healing)
            }
        });
    rsx! { {sections} }
}
