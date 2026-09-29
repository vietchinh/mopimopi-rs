//! Raid mode: one small card per player (colour strip, icon, name, DPS or HPS).

use super::table_environment::TableEnvironment;
use super::visible_players::visible_players;
use crate::domain::formatting::{cell_fragments, cell_fragments_ranked};
use crate::domain::settings::Settings;
use crate::infrastructure::act::data::{CombatantRecord, EncounterRecord};
use crate::presentation::ui::shared::palette::player_bar_color;
use crate::presentation::ui::shared::row_identity::row_element_id;
use crate::presentation::ui::shared::style_values::StyleValues;
use crate::presentation::ui::shared::text_display::{job_icon_view, text_fragments_view};
use dioxus::prelude::*;

/// Every variable shared by every card in this grid: card background/border/height, own-row vs.
/// other-rows text (both sets -- each card just picks which class to use), sizes, and how many
/// cards fit per row. Set once, on the outer grid container.
fn grid_style(settings: &Settings) -> String {
    let values = StyleValues::new(settings);
    let line = format!("{} solid {}", values.slider_as_rem("sizeLine"), values.color_with_opacity("tableLine", "tableLine"));
    let font = values.font_stack("fBody", "'Segoe UI', 'sans-serif'");
    let (bold_own, bold_other) = (values.bold_or_normal("boldYOU"), values.bold_or_normal("boldOther"));
    let body_style = values.italic_or_normal("body_italic");
    let cards_per_row = values.slider("size24TableSlice").max(1.0);
    format!(
        "--raid-cell-border:{line};--raid-text-font:{font};--raid-text-style:{body_style};--raid-cards-per-row:{cards_per_row};\
         --raid-cell-bg-other:{};--raid-cell-bg-own:{};--raid-cell-height:{};\
         --raid-text-other-weight:{bold_other};--raid-text-other-color:#{};--raid-text-other-opacity:{};--raid-text-other-shadow:{};\
         --raid-text-own-weight:{bold_own};--raid-text-own-color:#{};--raid-text-own-opacity:{};--raid-text-own-shadow:{};\
         --raid-name-size:{};--raid-data-size:{};--raid-idx-width:{};--raid-idx-opacity:{};--raid-icon-width:{}",
        values.color_with_opacity("view24BgOther", "view24BgOther"),
        values.color_with_opacity("view24BgYOU", "view24BgYOU"),
        values.slider_as_rem("size24TableHeight"),
        values.color("view24TableOther"),
        values.opacity("view24TableOther"),
        values.text_outline("Other"),
        values.color("view24TableYOU"),
        values.opacity("view24TableYOU"),
        values.text_outline("YOU"),
        values.slider_as_rem("size24BodyNameText"),
        values.slider_as_rem("size24BodyDataText"),
        values.slider_as_rem("size24TableIdxWd"),
        values.opacity("bar"),
        values.slider_as_rem("size24BodyIcon"),
    )
}

pub(super) fn raid_grid(environment: &TableEnvironment, combatants: &[CombatantRecord], encounter: &EncounterRecord, is_healing: bool) -> Element {
    let players = visible_players(environment.settings, combatants, is_healing);
    let cards_per_row = (environment.settings.slider_value("size24TableSlice") as usize).max(1);
    let value_column = if is_healing { "enchps" } else { "encdps" };
    let label = super::table_label(is_healing);
    let suffix = environment.element_id_suffix;
    let grid_vars = grid_style(environment.settings);

    let rows = players.chunks(cards_per_row).enumerate().map(|(row_index, row_players)| {
        let is_first_row = row_index == 0;
        let last_index = row_players.len().saturating_sub(1);
        let cards = row_players.iter().enumerate().map(|(index, player)| raid_card(environment, player.combatant, player.rank, encounter, value_column, is_first_row, index == last_index));
        // The card count is one value for the whole grid, so it is the `--raid-cards-per-row`
        // variable (set once on the container) read by `raid-row`, not a style on every row.
        rsx! { div { class: "rRow raid-row", {cards} } }
    });
    // Raid mode has no header table, but the original still has the (empty) header `div` of each table,
    // and it is that element that carries the "Spacing of DPS/HPS Table" margin: without it the HPS grid
    // would sit against the end of the DPS grid.
    let table_gap = StyleValues::new(environment.settings).slider_as_rem(if is_healing { "sizeHPSGap" } else { "sizeDPSGap" });
    rsx! {
        div { id: "{label}Header{suffix}", margin_top: "{table_gap}", div { id: "{label}oldHeader{suffix}" } }
        div { id: "{label}Body{suffix}", style: "{grid_vars}", {rows} }
    }
}

fn raid_card(environment: &TableEnvironment, combatant: &CombatantRecord, rank: usize, encounter: &EncounterRecord, value_column: &str, is_first_row: bool, is_last_in_row: bool) -> Element {
    let row_id = row_element_id(&combatant.name);
    let color_strip = player_bar_color(environment.settings, combatant, &row_id);
    let icon = job_icon_view(environment.settings, combatant, false);
    let name = text_fragments_view(cell_fragments_ranked("name", combatant, encounter, &environment.cell_context, rank), false);
    let value = text_fragments_view(cell_fragments(value_column, combatant, encounter, &environment.cell_context), false);
    let is_own = row_id == "YOU";
    // The first row's top border and each row's last card's right border are set inline: the original's
    // stylesheet has more specific rules for both (`.rRow:first-child .rCell`, `.rRow .rCell:last-child`)
    // that a class here would lose to, and the original sets them inline too.
    let card_style = if is_own { "raid-cell-own" } else { "raid-cell-other" };
    rsx! {
        table {
            key: "{row_id}",
            id: "{row_id}",
            class: "rCell {card_style}",
            border_right: if is_last_in_row { "var(--raid-cell-border)" },
            width: "100%",
            border_top: if is_first_row { "var(--raid-cell-border)" },
            tbody {
                tr {
                    td { class: "rIdx raid-idx", rowspan: "2", background: "{color_strip}", opacity: "var(--raid-idx-opacity)" }
                    td { class: "rIcon raid-icon", {icon} }
                    td { class: "rName raid-name", {name} }
                }
                tr { td { class: "rData raid-data", colspan: "2", {value} } }
            }
        }
    }
}
