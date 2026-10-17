//! The normal table: a header row and one row per participant.
//!
//! Styling here is computed inline, once per table (not per row/cell), and set on the header and
//! body wrapper elements -- every descendant (rows, cells, bars, icons) inherits the relevant
//! variables through the normal CSS cascade rather than recomputing or redeclaring them. Only two
//! things are genuinely per-element: which of the `table-text-own`/`table-text-other` classes a
//! row picks (its own vs. everyone else's, real per-row data), and each cell's own
//! width/padding/align (a `Column`, genuinely per-column data).

use super::cell_classes::{BODY_CELL_BORDER, HEADER_CELL, HEADER_TABLE};
use super::graph_bars::graph_bars;
use super::table_environment::SharedEnvironment;
use super::visible_participants::visible_participants;
use crate::application::app_state::TablesContext;
use crate::domain::formatting::{cell_fragments_ranked, CellContext};
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use crate::domain::combat::{EncounterFacts, Participant};
use crate::presentation::ui::shared::row_identity::row_element_id;
use crate::presentation::ui::areas::Column;
use crate::presentation::ui::shared::text_display::{job_icon_view, text_fragments_view};
use dioxus::prelude::*;
use std::rc::Rc;
use std::collections::HashSet;

pub(super) fn standard_table(
    environment: &SharedEnvironment,
    participants: &[Rc<Participant>],
    encounter: &Rc<EncounterFacts>,
    is_healing: bool,
    blurred_rows: &HashSet<String>,
) -> Element {
    let table = &environment.table;
    let shown = visible_participants(&table.kind(is_healing).filter, participants, is_healing);
    // The best value in this table; bar widths are relative to it. A plain scan, not the position
    // of the first row, since a healing table's own order (sorted in `visible_participants`, best first)
    // would make that valid too, but a damage table's order is only usually already sorted this way.
    let top_value = participants
        .iter()
        .map(|participant| if is_healing { participant.healed } else { participant.damage }.value())
        .fold(0.0, f64::max);
    let label = super::table_label(is_healing);
    let columns = environment.columns.of(is_healing);
    let suffix = environment.element_id_suffix;
    let body_height = table.body_height_rem(is_healing, shown.len());

    // Everything below is worked out once per table, not per row: the variables the whole table
    // shares, and the bar settings.
    let header = table.header_vars();
    let body = table.body_vars();
    // "Spacing of DPS/HPS Table": the space above each table's header.
    let table_gap = table.kind(is_healing).gap;
    let rows = shown.iter().map(|ranked| {
        let row_id = row_element_id(&ranked.participant.name);
        let blur_key = format!("{label}{row_id}");
        let is_blurred = blurred_rows.contains(&blur_key);
        rsx! {
            ParticipantRow {
                key: "{blur_key}",
                environment: environment.clone(),
                participant: Rc::clone(ranked.participant),
                encounter: Rc::clone(encounter),
                rank: ranked.rank,
                top_value,
                is_healing,
                is_blurred,
            }
        }
    });

    rsx! {
        // No header for a table with no rows (checked against the original: its header only appears with rows).
        if !shown.is_empty() {
            div { id: "{label}Header{suffix}", style: "{header}margin-top:{table_gap}",
                div { id: "{label}oldHeader{suffix}", TableHeader { columns: columns.to_vec() } }
            }
        }
        div {
            id: "{label}Body{suffix}",
            style: if shown.is_empty() { "{body}" } else { "{body}height:{body_height}rem" },
            div { id: "{label}oldBody{suffix}", {rows} }
        }
    }
}

/// The column titles. A component so that it is drawn again only when the columns (or the language of a tooltip) change.
#[component]
fn TableHeader(columns: Vec<Column>) -> Element {
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

/// One participant's row: the cells, then the bars behind them. A component, so a row whose data and settings did not change is
/// not drawn again when another row's did.
#[component]
fn ParticipantRow(
    environment: SharedEnvironment,
    participant: Rc<Participant>,
    encounter: Rc<EncounterFacts>,
    rank: usize,
    top_value: f64,
    is_healing: bool,
    is_blurred: bool,
) -> Element {
    let tables = use_context::<TablesContext>();
    let label = super::table_label(is_healing);
    let row_id = row_element_id(&participant.name);
    let blur_key = format!("{label}{row_id}");
    let is_own = participant.is_yours_to_highlight();
    let columns = environment.columns.of(is_healing);
    let cell_context = environment.cell_context();
    let cells = columns.iter().map(|column| {
        if column.name == "Class" {
            job_icon_cell(&environment, &participant, column, tables.blurred_player_rows, blur_key.clone())
        } else {
            cell(&cell_context, &participant, rank, &encounter, column, is_blurred)
        }
    });

    rsx! {
        div {
            id: "{row_id}",
            class: "tableWrap chrome-row themed-text",
            class: if is_own { "text-own" },
            class: if participant.is_your_pet() { "myPet" },
            table { class: "tableBody", tbody { tr { {cells} } } }
            {graph_bars(&environment, &participant, top_value, is_healing, &row_id)}
            div { class: "barBg bg-(--chrome-bar-bg) corner-body" }
        }
    }
}

fn cell(cell_context: &CellContext, participant: &Participant, rank: usize, encounter: &EncounterFacts, column: &Column, row_is_blurred: bool) -> Element {
    let is_blurred = column.name == "name" && row_is_blurred;
    let content = text_fragments_view(cell_fragments_ranked(&column.key, participant, encounter, cell_context, rank), true);
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
fn job_icon_cell(
    environment: &SharedEnvironment,
    participant: &Participant,
    column: &Column,
    blurred_player_rows: Signal<HashSet<String>>,
    blur_key: String,
) -> Element {
    let can_blur = environment.can_blur_names;
    let icon = job_icon_view(&environment.table.icon_set, participant, true, environment.npc_look());
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
                    let mut blurred_rows = blurred_player_rows;
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
