//! Raid mode: one small card per player (colour strip, icon, name, DPS or HPS).

use super::table_environment::SharedEnvironment;
use super::visible_players::visible_players;
use crate::domain::formatting::{cell_fragments, cell_fragments_ranked, ColumnKey};
use crate::domain::combat::{EncounterFacts, Member};
use crate::presentation::ui::shared::row_identity::row_element_id;
use crate::presentation::ui::shared::text_display::{job_icon_view, text_fragments_view};
use dioxus::prelude::*;

pub(super) fn raid_grid(environment: &SharedEnvironment, members: &[Member], encounter: &EncounterFacts, is_healing: bool) -> Element {
    let players = visible_players(&environment.table.kind(is_healing).filter, members, is_healing);
    let cards_per_row = environment.raid.cards_in_a_row();
    let label = super::table_label(is_healing);
    let suffix = environment.element_id_suffix;
    let grid_vars = environment.raid.grid_vars();

    let rows = players.chunks(cards_per_row).map(|row_players| {
        let cards = row_players.iter().map(|player| {
            rsx! {
                RaidCard {
                    key: "{row_element_id(&player.member.name)}",
                    environment: environment.clone(),
                    member: player.member.clone(),
                    encounter: encounter.clone(),
                    rank: player.rank,
                    is_healing,
                }
            }
        });
        // The card count is one value for the whole grid, so it is the `--raid-cards-per-row`
        // variable (set once on the container) read by `raid-row`, not a style on every row.
        rsx! { div { class: "rRow raid-row", {cards} } }
    });
    // Raid mode has no header table, but the original still has the (empty) header `div` of each table,
    // and it is that element that carries the "Spacing of DPS/HPS Table" margin: without it the HPS grid
    // would sit against the end of the DPS grid.
    let table_gap = environment.table.kind(is_healing).gap;
    rsx! {
        div { id: "{label}Header{suffix}", margin_top: "{table_gap}", div { id: "{label}oldHeader{suffix}" } }
        div { id: "{label}Body{suffix}", style: "{grid_vars}", {rows} }
    }
}

/// One card of the raid grid. A component, so a card whose data and settings did not change is not drawn again.
#[component]
fn RaidCard(environment: SharedEnvironment, member: Member, encounter: EncounterFacts, rank: usize, is_healing: bool) -> Element {
    let value_column = if is_healing { ColumnKey::Enchps } else { ColumnKey::Encdps };
    let cell_context = environment.cell_context();
    let row_id = row_element_id(&member.name);
    let color_strip = environment.bar_color(&member, &row_id);
    let icon = job_icon_view(&environment.table.icon_set, &member, false, environment.npc_look());
    let name = text_fragments_view(cell_fragments_ranked(&ColumnKey::Name, &member, &encounter, &cell_context, rank), false);
    let value = text_fragments_view(cell_fragments(&value_column, &member, &encounter, &cell_context), false);
    let is_own = row_id == "YOU";
    let card_style = if is_own { "raid-cell-own" } else { "raid-cell-other" };
    rsx! {
        table {
            id: "{row_id}",
            class: "rCell {card_style} themed-text",
            class: if is_own { "text-own" },
            width: "100%",
            tbody {
                tr {
                    td { class: "rIdx w-(--raid-idx-width)", rowspan: "2", background: "{color_strip}", opacity: "var(--raid-idx-opacity)" }
                    td { class: "rIcon w-(--raid-icon-width) [&_img]:w-(--raid-icon-width)", {icon} }
                    td { class: "rName raid-name", {name} }
                }
                tr { td { class: "rData raid-data", colspan: "2", {value} } }
            }
        }
    }
}
