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
pub use graph_bars::BarHistory;
mod raid_grid;
pub(crate) mod cell_classes;
pub(crate) mod standard_table;
mod table_environment;
mod visible_players;

use crate::application::app_state::{SettingsContext, SettingsScreenContext, TablesContext};
use crate::presentation::ui::overlay_plugin_context::OverlayPluginContext;
use dioxus::prelude::*;
use crate::infrastructure::act::data::CombatDataMessage;
use table_environment::{SharedEnvironment, TableEnvironment};

/// "DPS" or "HPS": the prefix used in element ids, column flags and setting keys. There are only
/// ever these two tables, so a plain bool (`is_healing`) says which one is meant throughout this
/// module, rather than a dedicated enum for a distinction this local.
fn table_label(is_healing: bool) -> &'static str {
    if is_healing { "HPS" } else { "DPS" }
}

/// No tables: also the moment the bars forget where they were (the original clears its bar sizes when a fight is saved).
fn empty_tables() -> Element {
    if let Some(history) = try_consume_context::<BarHistory>() {
        history.forget_all();
    }
    rsx! {}
}

/// Shows the tables in the order and combination the user configured.
///
/// What every row shares (the settings that shape it, the player's name, whether a fight is running) is worked out in a memo, so
/// it is rebuilt, and the rows are told, only when one of those really changed. A row is a component: when its own data and
/// that shared environment are unchanged it is not drawn again (see `PlayerRow` and `RaidCard`).
#[component]
pub fn CombatTables(is_settings_preview: bool) -> Element {
    let overlay_plugin_context = use_context::<OverlayPluginContext>();
    let settings_context = use_context::<SettingsContext>();
    let tables = use_context::<TablesContext>();
    let settings_screen = use_context::<SettingsScreenContext>();
    let view = use_context::<crate::presentation::ui::areas::SettingsView>();

    // Its own memo, so a message that does not change whether a fight is running does not rebuild the environment.
    let fight_is_running = use_memo(use_reactive!(|is_settings_preview| is_settings_preview || overlay_plugin_context.get_is_encounter_active()));
    let environment = use_memo(use_reactive!(|is_settings_preview| {
        SharedEnvironment::new(TableEnvironment::new(
            *fight_is_running.read(),
            &settings_context.settings.read(),
            &view.table.read(),
            &view.columns.read(),
            &view.bars.read(),
            &view.raid.read(),
            &tables.local_player_name.read(),
            is_settings_preview,
        ))
    }));
    let merge_pets = view.page.read().merge_pets;

    // The real tables draw whatever is currently displayed (live, a history entry, or frozen
    // while settings are open); the settings preview always draws the built-in sample fight.
    let displayed = if is_settings_preview { None } else { tables.displayed_combat_data.read().clone() };
    let message: &CombatDataMessage = if is_settings_preview {
        crate::application::app_state::sample_combat_message(merge_pets)
    } else {
        let Some(message) = displayed.as_deref() else { return empty_tables() };
        message
    };
    let (combatants, encounter) = (&message.combatants, &message.encounter);

    // The original only builds its tables when the data has a combatant named "YOU" (`onCombatDataUpdate`
    // starts with that check); without one both tables stay empty.
    if !combatants.iter().any(|combatant| combatant.name == "YOU") {
        return empty_tables();
    }

    let environment = environment();
    let raid_mode = environment.table.raid_mode(combatants.len()) || (is_settings_preview && *settings_screen.settings_preview_raid_mode.read());
    let blurred_rows = tables.blurred_player_rows.read();

    let sections = environment.table.tables_in_order().into_iter().map(|is_healing| {
        if raid_mode {
            raid_grid::raid_grid(&environment, combatants, encounter, is_healing)
        } else {
            standard_table::standard_table(&environment, combatants, encounter, is_healing, &blurred_rows)
        }
    });
    rsx! { {sections} }
}
