//! The normal table: a header row and one row per player.

use super::graph_bars::graph_bars;
use super::table_environment::TableEnvironment;
use super::visible_players::visible_players;
use crate::domain::combat::{EncounterRanking, Player, TableKind, COMBATANT_JOB_CODE, PET_JOB_CODE};
use crate::domain::formatting::cell_fragments;
use crate::presentation::theme::table_body_height_rem;
use crate::domain::translations::{translate, translations};
use crate::presentation::ui::shared::row_identity::row_element_id;
use crate::presentation::ui::shared::text_display::{job_icon_view, text_fragments_view};
use dioxus::prelude::*;

pub(super) fn standard_table(environment: &TableEnvironment, ranking: &EncounterRanking, table: TableKind) -> Element {
    let players = visible_players(environment.settings, ranking, table);
    let columns = environment.settings.column_order(table.short_label());
    let label = table.short_label();
    let suffix = environment.element_id_suffix;
    let body_height = table_body_height_rem(environment.settings, label, players.len());
    let rows = players.iter().map(|player| player_row(environment, ranking, table, player, &columns));

    rsx! {
        if !players.is_empty() {
            div { id: "{label}Header{suffix}",
                div { id: "{label}oldHeader{suffix}", {header_table(environment, &columns)} }
            }
        }
        div { id: "{label}Body{suffix}", style: if players.is_empty() { "" } else { "height:{body_height}rem" },
            div { id: "{label}oldBody{suffix}", {rows} }
        }
    }
}

fn header_table(environment: &TableEnvironment, columns: &[String]) -> Element {
    let language = environment.settings.language_code();
    let context = environment.context;
    let cells = columns.iter().map(|column| {
        let title = environment.settings.column_text(column, "tt");
        // Explanation shown as a tooltip (from the dictionary), if there is one for this column.
        let hint = {
            let entry = &translations().dictionary[column.as_str()]["tt"];
            if entry.is_null() { None } else { Some(translate(entry, &language)) }
        };
        rsx! {
            td {
                key: "{column}",
                class: "{column} cell",
                "{title}"
            }
        }
    });
    rsx! { table { class: "tableHeader", tbody { tr { {cells} } } } }
}

fn player_row(environment: &TableEnvironment, ranking: &EncounterRanking, table: TableKind, player: &Player, columns: &[String]) -> Element {
    let row_id = row_element_id(&player.name);
    let blur_key = format!("{}{row_id}", table.short_label());
    let is_local_players_pet = (player.job_code == PET_JOB_CODE || player.job_code == COMBATANT_JOB_CODE) && row_id.contains("YOU");
    let cells = columns
        .iter()
        .filter(|column| environment.settings.column_enabled_in_table(column, table.short_label()))
        .map(|column| cell(environment, ranking, player, column, &blur_key));

    rsx! {
        div {
            key: "{blur_key}",
            id: "{row_id}",
            class: if is_local_players_pet { "tableWrap myPet" } else { "tableWrap" },
            table { class: "tableBody", tbody { tr { {cells} } } }
            {graph_bars(environment, ranking, table, player, &row_id)}
            div { class: "barBg" }
        }
    }
}

fn cell(environment: &TableEnvironment, ranking: &EncounterRanking, player: &Player, column: &str, blur_key: &str) -> Element {
    if column == "Class" {
        return job_icon_cell(environment, player, blur_key);
    }
    let is_blurred = column == "name" && environment.blurred_rows.contains(blur_key);
    let class = if is_blurred { format!("{column} cell hidden") } else { format!("{column} cell") };
    let content = text_fragments_view(cell_fragments(column, player, ranking, &environment.cell_context));
    rsx! { td { key: "{column}", class: "{class}", {content} } }
}

/// The job icon cell. Clicking it blurs / unblurs the player's name (only outside of fights).
fn job_icon_cell(environment: &TableEnvironment, player: &Player, blur_key: &str) -> Element {
    let context = environment.context;
    let can_blur = environment.can_blur_names;
    let blur_key = blur_key.to_string();
    let icon = job_icon_view(environment.settings, player);
    rsx! {
        td {
            key: "{blur_key}",
            class: "Class cell",
            style: if can_blur { "cursor:pointer" } else { "" },
            onclick: move |_| {
                if can_blur {
                    let mut blurred_rows = context.blurred_player_rows;
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
