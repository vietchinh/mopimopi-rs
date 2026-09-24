//! The DPS and HPS tables (rows with graph bars) and the compact "raid mode" grid.
//!
//! Reading order: `CombatTables` (entry) -> `standard_table` / `raid_grid` -> `player_row` -> `graph_bars`.
//!
//! * `table_environment`  – everything the drawing functions share
//! * `visible_players`    – which players a table shows
//! * `standard_table`     – header, body and one row per player
//! * `graph_bars`         – the coloured bars behind a row
//! * `raid_grid`          – the compact cards

mod graph_bars;
mod raid_grid;
mod standard_table;
mod table_environment;
mod visible_players;

use crate::application::app_state::AppContext;
use crate::domain::combat::TableKind;
use crate::presentation::ui::shared::rankings_source::rankings_to_draw;
use dioxus::prelude::*;
use table_environment::TableEnvironment;

/// Shows the tables in the order and combination the user configured.
#[component]
pub fn CombatTables(is_settings_preview: bool) -> Element {
    let context = use_context::<AppContext>();
    let Some(rankings) = rankings_to_draw(context, is_settings_preview) else { return rsx! {} };
    if rankings.by_damage.local_player().is_none() {
        return rsx! {}; // like the original, nothing is drawn until the local player's row exists
    }
    let settings = context.settings.read();
    let local_player_name = context.local_player_name.read().clone();
    let blurred_rows = context.blurred_player_rows.read().clone();
    let environment = TableEnvironment::new(context, &settings, &local_player_name, &blurred_rows, is_settings_preview);

    let raid_mode = (settings.option_enabled("view24") && rankings.by_damage.party_size as f64 >= settings.option_number("view24_Number"))
        || (is_settings_preview && *context.settings_preview_raid_mode.read());
    let damage_table_first = settings.option_number("tableOrder") as i32 == 1;
    let tables = if damage_table_first { [TableKind::Damage, TableKind::Healing] } else { [TableKind::Healing, TableKind::Damage] };

    let sections = tables
        .into_iter()
        .filter(|table| settings.option_enabled(&format!("view{}", table.short_label())))
        .map(|table| {
            let ranking = rankings.ranking_for_table(table);
            if raid_mode {
                raid_grid::raid_grid(&environment, ranking, table)
            } else {
                standard_table::standard_table(&environment, ranking, table)
            }
        });
    rsx! { {sections} }
}
