//! The normal table: a header row and one row per player.
//!
//! Styling here is computed inline, once per table (not per row/cell), and set on the header and
//! body wrapper elements -- every descendant (rows, cells, bars, icons) inherits the relevant
//! variables through the normal CSS cascade rather than recomputing or redeclaring them. Only two
//! things are genuinely per-element: which of the `table-text-own`/`table-text-other` classes a
//! row picks (its own vs. everyone else's, real per-row data), and each cell's own
//! width/padding/align (a `Column`, genuinely per-column data).

use super::cell_classes::{BODY_CELL_BORDER, HEADER_CELL, HEADER_TABLE};
use super::graph_bars::graph_bars;
use super::table_environment::TableEnvironment;
use super::visible_players::visible_players;
use crate::application::app_state::AppContext;
use crate::domain::combat::CombatantKind;
use crate::domain::formatting::cell_fragments_ranked;
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use crate::infrastructure::act::data::{CombatantRecord, EncounterRecord};
use crate::presentation::ui::shared::row_identity::row_element_id;
use crate::presentation::ui::areas::Column;
use crate::presentation::ui::shared::text_display::{job_icon_view, text_fragments_view};
use dioxus::prelude::*;

pub(super) fn standard_table(environment: &TableEnvironment, combatants: &[CombatantRecord], encounter: &EncounterRecord, is_healing: bool) -> Element {
    let table = environment.table;
    let players = visible_players(&table.kind(is_healing).filter, combatants, is_healing);
    // The best value in this table; bar widths are relative to it. A plain scan, not the position
    // of the first row, since a healing table's own order (sorted in `visible_players`, best first)
    // would make that valid too, but a damage table's order is only usually already sorted this way.
    let top_value = combatants
        .iter()
        .map(|combatant| if is_healing { combatant.healed } else { combatant.damage })
        .fold(0.0, f64::max);
    let label = super::table_label(is_healing);
    let columns = environment.columns.of(is_healing);
    let suffix = environment.element_id_suffix;
    let body_height = table.body_height_rem(is_healing, players.len());

    // Everything below is worked out once per table, not per row: the variables the whole table
    // shares, and the bar settings.
    let header = table.header_vars();
    let body = table.body_vars();
    // "Spacing of DPS/HPS Table": the space above each table's header.
    let table_gap = table.kind(is_healing).gap;
    let rows = players
        .iter()
        .map(|player| player_row(environment, player.combatant, player.rank, top_value, encounter, is_healing, columns));

    rsx! {
        // No header for a table with no rows (checked against the original: its header only appears with rows).
        if !players.is_empty() {
            div { id: "{label}Header{suffix}", style: "{header}margin-top:{table_gap}",
                div { id: "{label}oldHeader{suffix}", {header_table(columns)} }
            }
        }
        div {
            id: "{label}Body{suffix}",
            style: if players.is_empty() { "{body}" } else { "{body}height:{body_height}rem" },
            div { id: "{label}oldBody{suffix}", {rows} }
        }
    }
}

fn header_table(columns: &[Column]) -> Element {
    let cells = columns.iter().map(|column| {
        // Explanation shown as a tooltip (from the dictionary), if there is one for this column.
        let hint = {
            let entry = &translations().dictionary[column.name.as_str()]["tt"];
            if entry.is_null() { None } else { Some(translate(entry)) }
        };
        rsx! {
            td {
                key: "{column.name}",
                // `first:`/`last:` are Tailwind's `:first-child`/`:last-child` variants, so whichever
                // cell ends up first or last in the DOM gets the rounded corners even though columns
                // are user-reorderable; `not-last:` is the built-in negation of `last:`.
                class: "{column.name} cell{HEADER_CELL}",
                width: "{column.width}",
                padding: "{column.padding_css()}",
                text_align: "{column.header_align.as_str()}",
                title: hint,
                "{column.title}"
            }
        }
    });
    rsx! { table { class: HEADER_TABLE, tbody { tr { {cells} } } } }
}

#[allow(clippy::too_many_arguments)]
fn player_row(
    environment: &TableEnvironment,
    combatant: &CombatantRecord,
    rank: usize,
    top_value: f64,
    encounter: &EncounterRecord,
    is_healing: bool,
    columns: &[Column],
) -> Element {
    let label = super::table_label(is_healing);
    let row_id = row_element_id(&combatant.name);
    let blur_key = format!("{label}{row_id}");
    let is_local_players_pet = matches!(combatant.kind(), CombatantKind::Pet { owner_name, .. } if owner_name == "YOU");
    let is_own = row_id == "YOU" || is_local_players_pet;
    let cells = columns.iter().map(|column| cell(environment, combatant, rank, encounter, column, &blur_key));

    rsx! {
        div {
            key: "{blur_key}",
            id: "{row_id}",
            class: "tableWrap chrome-row themed-text",
            class: if is_own { "text-own" },
            class: if is_local_players_pet { "myPet" },
            table { class: "tableBody", tbody { tr { {cells} } } }
            {graph_bars(environment, combatant, top_value, is_healing, &row_id)}
            div { class: "barBg bg-(--chrome-bar-bg) corner-body" }
        }
    }
}

fn cell(environment: &TableEnvironment, combatant: &CombatantRecord, rank: usize, encounter: &EncounterRecord, column: &Column, blur_key: &str) -> Element {
    if column.name == "Class" {
        return job_icon_cell(environment, combatant, blur_key, column);
    }
    let is_blurred = column.name == "name" && environment.blurred_rows.contains(blur_key);
    let content = text_fragments_view(cell_fragments_ranked(&column.name, combatant, encounter, &environment.cell_context, rank), true);
    rsx! {
        td {
            key: "{column.name}",
            class: "{column.name} cell{BODY_CELL_BORDER}",
            width: "{column.width}",
            padding: "{column.padding_css()}",
            text_align: "{column.body_align.as_str()}",
            display: if is_blurred { "none" },
            {content}
        }
    }
}

/// The job icon cell. Clicking it blurs / unblurs the player's name (only outside of fights).
fn job_icon_cell(environment: &TableEnvironment, combatant: &CombatantRecord, blur_key: &str, column: &Column) -> Element {
    let can_blur = environment.can_blur_names;
    let blur_key = blur_key.to_string();
    let icon = job_icon_view(&environment.table.icon_set, combatant, true);
    rsx! {
        td {
            key: "{column.name}",
            class: "Class cell{BODY_CELL_BORDER}",
            width: "{column.width}",
            padding: "{column.padding_css()}",
            text_align: "{column.body_align.as_str()}",
            cursor: if can_blur { "pointer" },
            onclick: move |_| {
                if can_blur {
                    // `consume_context` is a plain function; `use_context` is a hook and must not be called once per row.
                    let mut blurred_rows = consume_context::<AppContext>().blurred_player_rows;
                    let mut rows = blurred_rows.write();
                    if !rows.remove(&blur_key) {
                        rows.insert(blur_key.clone());
                    }
                }
            },
            {icon}
        }
    }
}
