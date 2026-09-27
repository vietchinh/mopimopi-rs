//! Raid mode: one small card per player (colour strip, icon, name, DPS or HPS).

use super::table_environment::TableEnvironment;
use crate::domain::combat::{TableKind};
use crate::domain::formatting::cell_fragments;
use crate::presentation::ui::shared::palette::player_bar_color;
use crate::presentation::ui::shared::row_identity::row_element_id;
use crate::presentation::ui::shared::text_display::{job_icon_view, text_fragments_view};
use dioxus::prelude::*;
use crate::infrastructure::act::data::{CombatantRecord, EncounterRecord};

pub(super) fn raid_grid(environment: &TableEnvironment, combatant_records: &Vec<CombatantRecord>, encounter_record: &EncounterRecord, table: TableKind) -> Element {
    // let players = visible_players(environment.settings, table);
    let cards_per_row = (environment.settings.slider_value("size24TableSlice") as usize).max(1);
    let value_column = match table {
        TableKind::Damage => "encdps",
        TableKind::Healing => "enchps",
    };
    let label = table.short_label();
    let suffix = environment.element_id_suffix;

    let rows = combatant_records.chunks(cards_per_row).map(|combatant_records| {
        let cards = combatant_records.iter().map(|combatant_record| raid_card(environment, combatant_record, encounter_record,  value_column));
        rsx! { div { class: "rRow", {cards} } }
    });
    rsx! { div { id: "{label}Body{suffix}", {rows} } }
}

fn raid_card(environment: &TableEnvironment, combatant_record: &CombatantRecord, encounter_record: &EncounterRecord, value_column: &str) -> Element {
    let row_id = row_element_id(&combatant_record.name);
    let color_strip = player_bar_color(environment.settings, combatant_record, &row_id);
    let icon = job_icon_view(environment.settings, combatant_record);
    let name = text_fragments_view(cell_fragments("name", combatant_record, encounter_record, &environment.cell_context));
    let value = text_fragments_view(cell_fragments(value_column, combatant_record, encounter_record, &environment.cell_context));
    rsx! {
        table { key: "{row_id}", id: "{row_id}", class: "rCell",
            tbody {
                tr {
                    td { class: "rIdx", rowspan: "2", style: "background:{color_strip}" }
                    td { class: "rIcon", {icon} }
                    td { class: "rName", {name} }
                }
                tr { td { class: "rData", colspan: "2", {value} } }
            }
        }
    }
}
