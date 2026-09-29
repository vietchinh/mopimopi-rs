//! The normal table: a header row and one row per player.
//!
//! Styling here is computed inline, once per table (not per row/cell), and set on the header and
//! body wrapper elements -- every descendant (rows, cells, bars, icons) inherits the relevant
//! variables through the normal CSS cascade rather than recomputing or redeclaring them. Only two
//! things are genuinely per-element: which of the `table-text-own`/`table-text-other` classes a
//! row picks (its own vs. everyone else's, real per-row data), and each cell's own
//! width/padding/align (`column_style`, genuinely per-column data).

use super::graph_bars::{graph_bar_radius, graph_bars, BarSettings};
use super::table_body_height_rem;
use super::table_environment::TableEnvironment;
use super::visible_players::visible_players;
use crate::application::app_state::AppContext;
use crate::domain::combat::CombatantKind;
use crate::domain::formatting::cell_fragments_ranked;
use crate::domain::settings::Settings;
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use crate::infrastructure::act::data::{CombatantRecord, EncounterRecord};
use crate::presentation::ui::shared::row_identity::row_element_id;
use crate::presentation::ui::shared::style_values::{rem, StyleValues};
use crate::presentation::ui::shared::text_display::{job_icon_view, text_fragments_view};
use dioxus::prelude::*;
use std::collections::HashMap;

/// "Coverage" setting: which element the table's rounded corners are applied to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CornerCoverage {
    HeaderOnly,
    BodyOnly,
    HeaderAndBody,
}

impl CornerCoverage {
    fn from_option_number(number: i32) -> CornerCoverage {
        match number {
            1 => CornerCoverage::HeaderOnly,
            2 => CornerCoverage::BodyOnly,
            _ => CornerCoverage::HeaderAndBody,
        }
    }
}

/// Every variable shared by every header cell in this table, as one inline `style` for the
/// wrapper (every cell inherits it): corner radii, height, font size and the header chrome
/// (background, text, cell borders). Set once per table and read by the static classes in
/// `tailwind.css`, rather than formatted onto each cell -- what the Dioxus docs recommend
/// (custom properties instead of per-element string formatting).  Also used directly by the History screen (`history_screen/mod.rs`),
/// whose header needs identical styling. The corner radius is zeroed out when `applyScope`
/// doesn't cover the header, rather than choosing whether to emit a rule the way the
/// pre-Tailwind version did.
pub(crate) fn header_style(values: &StyleValues) -> String {
    let scope = CornerCoverage::from_option_number(values.number("applyScope") as i32);
    let header_on = matches!(scope, CornerCoverage::HeaderOnly | CornerCoverage::HeaderAndBody);
    let [tl, tr, bl, br] = if header_on {
        values.corner_radius("rd_table", "sizeRadiusTable")
    } else {
        ["0rem".to_string(), "0rem".to_string(), "0rem".to_string(), "0rem".to_string()]
    };
    let header_color = values.color_with_opacity("tableHd", "tableHd");
    let vertical_line_width = values.slider_as_rem("sizeLineVer");
    let style = format!(
        "--table-radius-header-tl:{tl};--table-radius-header-tr:{tr};--table-radius-header-bl:{bl};--table-radius-header-br:{br};\
         --chrome-header-height:{};--chrome-header-text-size:{};\
         --chrome-header-margin:{};--chrome-header-bg:{header_color};--chrome-header-color:{};--chrome-header-font:{};\
         --chrome-header-cell-border:{vertical_line_width} solid {header_color};\
         --column-header-style:{}",
        values.slider_as_rem("sizeHd"),
        values.slider_as_rem("sizeHdText"),
        values.slider_as_rem("sizeHdGap"),
        values.color_with_opacity("tableHdText", "tableHdText"),
        values.font_stack("fHd", "'Roboto Condensed', 'sans-serif'"),
        values.italic_or_normal("header_italic"),
    );
    style
}

/// Every variable shared by every row in this table's body: row/bar chrome, own-row vs.
/// other-rows text (both sets, always -- each row just picks which class to use), and the two
/// globally-italic column settings. Also used directly by the History screen, which is always
/// "other" (it has no concept of "own row"). The corner radius is zeroed out the same way
/// `header_style` does above.
pub(crate) fn body_style(values: &StyleValues) -> String {
    let scope = CornerCoverage::from_option_number(values.number("applyScope") as i32);
    let body_on = matches!(scope, CornerCoverage::BodyOnly | CornerCoverage::HeaderAndBody);
    let [tl, tr, bl, br] = if body_on {
        values.corner_radius("rd_table", "sizeRadiusTable")
    } else {
        ["0rem".to_string(), "0rem".to_string(), "0rem".to_string(), "0rem".to_string()]
    };

    let horizontal_line = format!("{} solid {}", values.slider_as_rem("sizeLine"), values.color_with_opacity("tableLine", "tableLine"));
    let vertical_line_width = values.slider_as_rem("sizeLineVer");

    let font = values.font_stack("fBody", "'Segoe UI', 'sans-serif'");
    let size = values.slider_as_rem("sizeBodyText");
    let (bold_own, bold_other) = (values.bold_or_normal("boldYOU"), values.bold_or_normal("boldOther"));

    // Row height, icon width and dimmed-text size, and the bars' size/opacity/corners: one value
    // per table, read by static classes (`chrome-row`, `chrome-icon`, `chrome-ex`,
    // `chrome-bar-*`) instead of being formatted onto every row, icon, span and bar.
    let row_height = values.slider("sizeBody");
    let bar = |kind: &str, height_key: &str, opacity_key: &str| {
        let height = values.slider(height_key);
        format!(
            "--chrome-bar-{kind}-height:{};--chrome-bar-{kind}-margin:{};--chrome-bar-{kind}-opacity:{};",
            rem(height),
            rem(row_height - height),
            values.opacity(opacity_key)
        )
    };
    let bars = format!(
        "{}{}{}{}--chrome-bar-radius:{};",
        bar("main", "sizeGraph_bar", "bar"),
        bar("pet", "sizeGraph_pet", "pet"),
        bar("ds", "sizeGraph_ds", "ds"),
        bar("oh", "sizeGraph_oh", "oh"),
        graph_bar_radius(values),
    );

    let style = format!(
        "--table-radius-body-tl:{tl};--table-radius-body-tr:{tr};--table-radius-body-bl:{bl};--table-radius-body-br:{br};\
         --chrome-row-height:{};--chrome-icon-size:{};--chrome-ex-size:{};{bars}\
         --chrome-row-border:{horizontal_line};--chrome-bar-bg:{};\
         --chrome-cell-border:{vertical_line_width} solid {};\
         --table-text-font:{font};--table-text-size:{size};\
         --table-text-own-color:#{};--table-text-own-opacity:{};--table-text-own-weight:{bold_own};--table-text-own-shadow:{};--table-text-own-ex-color:#{};\
         --table-text-other-color:#{};--table-text-other-opacity:{};--table-text-other-weight:{bold_other};--table-text-other-shadow:{};--table-text-other-ex-color:#{};\
         --column-body-style:{}",
        values.slider_as_rem("sizeBody"),
        values.slider_as_rem("sizeBodyIcon"),
        rem(values.slider("sizeBodyText") - 1.0),
        values.color_with_opacity("tableBg", "tableBg"),
        values.color_with_opacity("tableLineVer", "tableLineVer"),
        values.color("tableYOU"),
        values.opacity("tableYOU"),
        values.text_outline("YOU"),
        values.color("tableExYOU"),
        values.color("tableOther"),
        values.opacity("tableOther"),
        values.text_outline("Other"),
        values.color("tableExOther"),
        values.italic_or_normal("body_italic"),
    );
    style
}

/// The three genuinely per-column values, applied to a column's cells as typed CSS-property
/// attributes (`width:`, `padding:`, `text_align:`). It differs per *column*, over a set of columns
/// the user can reorder and switch on and off, so there is no fixed set of variables or classes to
/// point at -- but it is the same for every row, so `column_styles` builds it once per table.
struct ColumnStyle {
    width: String,
    padding: String,
    align: &'static str,
}

static NO_COLUMN_STYLE: ColumnStyle = ColumnStyle { width: String::new(), padding: String::new(), align: "" };

/// `align_field` is `"alignHeader"` or `"alignBody"` -- the only one of the three that differs
/// between a column's header cell and its body cells.
fn column_style(settings: &Settings, column: &str, align_field: &str) -> ColumnStyle {
    let width = if column == "name" { "100%".to_string() } else { rem(settings.column_number(column, "width")) };
    let align = match settings.column_text(column, align_field).as_str() {
        "left" => "left",
        "right" => "right",
        _ => "center",
    };
    ColumnStyle { width, padding: format!("0 {}", rem(settings.column_number(column, "padding"))), align }
}

/// `column_style` for every column of one table, built once and shared by all its rows (it does
/// not depend on the row) instead of being worked out again for every cell of every row.
fn column_styles(settings: &Settings, columns: &[String], align_field: &str) -> HashMap<String, ColumnStyle> {
    columns.iter().map(|column| (column.clone(), column_style(settings, column, align_field))).collect()
}

pub(super) fn standard_table(environment: &TableEnvironment, combatants: &[CombatantRecord], encounter: &EncounterRecord, is_healing: bool) -> Element {
    let players = visible_players(environment.settings, combatants, is_healing);
    // The best value in this table; bar widths are relative to it. A plain scan, not the position
    // of the first row, since a healing table's own order (sorted in `visible_players`, best first)
    // would make that valid too, but a damage table's order is only usually already sorted this way.
    let top_value = combatants
        .iter()
        .map(|combatant| if is_healing { combatant.healed } else { combatant.damage })
        .fold(0.0, f64::max);
    let label = super::table_label(is_healing);
    let columns = environment.settings.column_order(label);
    let suffix = environment.element_id_suffix;
    let body_height = table_body_height_rem(environment.settings, label, players.len());

    // Everything below is worked out once per table, not per row: the variables the whole table
    // shares, the column styles, and the bar settings.
    let values = StyleValues::new(environment.settings);
    let header = header_style(&values);
    let body = body_style(&values);
    // "Spacing of DPS/HPS Table" (`sizeDPSGap`/`sizeHPSGap`): the space above each table's header.
    let table_gap = values.slider_as_rem(if is_healing { "sizeHPSGap" } else { "sizeDPSGap" });
    let header_columns = column_styles(environment.settings, &columns, "alignHeader");
    let body_columns = column_styles(environment.settings, &columns, "alignBody");
    let bars = BarSettings::new(environment, &values, is_healing);
    let rows = players
        .iter()
        .map(|player| player_row(environment, player.combatant, player.rank, top_value, encounter, is_healing, &columns, &body_columns, &bars));

    rsx! {
        // No header for a table with no rows (checked against the original: its header only appears with rows).
        if !players.is_empty() {
            div { id: "{label}Header{suffix}", style: "{header};margin-top:{table_gap}",
                div { id: "{label}oldHeader{suffix}", {header_table(environment, &columns, &header_columns)} }
            }
        }
        div {
            id: "{label}Body{suffix}",
            style: if players.is_empty() { "{body}" } else { "{body};height:{body_height}rem" },
            div { id: "{label}oldBody{suffix}", {rows} }
        }
    }
}

fn header_table(environment: &TableEnvironment, columns: &[String], column_styles: &HashMap<String, ColumnStyle>) -> Element {
    let cells = columns.iter().map(|column| {
        let title = environment.settings.column_text(column, "tt");
        // Explanation shown as a tooltip (from the dictionary), if there is one for this column.
        let hint = {
            let entry = &translations().dictionary[column.as_str()]["tt"];
            if entry.is_null() { None } else { Some(translate(entry)) }
        };
        let column_style = column_styles.get(column).unwrap_or(&NO_COLUMN_STYLE);
        rsx! {
            td {
                key: "{column}",
                // `first:`/`last:` are Tailwind's `:first-child`/`:last-child` variants, so whichever
                // cell ends up first or last in the DOM gets the rounded corners even though columns
                // are user-reorderable; `not-last:` is the built-in negation of `last:`.
                class: "{column} cell col-cell-header chrome-header-cell first:corner-header-left last:corner-header-right not-last:[border-right:var(--chrome-header-cell-border)]",
                width: "{column_style.width}",
                padding: "{column_style.padding}",
                text_align: "{column_style.align}",
                title: hint,
                "{title}"
            }
        }
    });
    rsx! { table { class: "tableHeader chrome-header-table", tbody { tr { {cells} } } } }
}

#[allow(clippy::too_many_arguments)]
fn player_row(
    environment: &TableEnvironment,
    combatant: &CombatantRecord,
    rank: usize,
    top_value: f64,
    encounter: &EncounterRecord,
    is_healing: bool,
    columns: &[String],
    column_styles: &HashMap<String, ColumnStyle>,
    bars: &BarSettings,
) -> Element {
    let label = super::table_label(is_healing);
    let row_id = row_element_id(&combatant.name);
    let blur_key = format!("{label}{row_id}");
    let is_local_players_pet = matches!(combatant.kind(), CombatantKind::Pet { owner_name, .. } if owner_name == "YOU");
    let is_own = row_id == "YOU" || is_local_players_pet;
    let text_style = if is_own { "table-text-own" } else { "table-text-other" };
    let cells = columns
        .iter()
        .filter(|column| environment.settings.column_enabled_in_table(column, label))
        .map(|column| cell(environment, combatant, rank, encounter, column, &blur_key, column_styles));

    rsx! {
        div {
            key: "{blur_key}",
            id: "{row_id}",
            class: "tableWrap chrome-row",
            class: "{text_style}",
            class: if is_local_players_pet { "myPet" },
            table { class: "tableBody", tbody { tr { {cells} } } }
            {graph_bars(environment, combatant, top_value, is_healing, &row_id, bars)}
            div { class: "barBg chrome-bar-bg corner-body" }
        }
    }
}

fn cell(
    environment: &TableEnvironment,
    combatant: &CombatantRecord,
    rank: usize,
    encounter: &EncounterRecord,
    column: &str,
    blur_key: &str,
    column_styles: &HashMap<String, ColumnStyle>,
) -> Element {
    if column == "Class" {
        return job_icon_cell(environment, combatant, blur_key, column_styles);
    }
    let is_blurred = column == "name" && environment.blurred_rows.contains(blur_key);
    let column_style = column_styles.get(column).unwrap_or(&NO_COLUMN_STYLE);
    let content = text_fragments_view(cell_fragments_ranked(column, combatant, encounter, &environment.cell_context, rank), true);
    rsx! {
        td {
            key: "{column}",
            class: "{column} cell col-cell-body not-last:[border-right:var(--chrome-cell-border)]",
            width: "{column_style.width}",
            padding: "{column_style.padding}",
            text_align: "{column_style.align}",
            display: if is_blurred { "none" },
            {content}
        }
    }
}

/// The job icon cell. Clicking it blurs / unblurs the player's name (only outside of fights).
fn job_icon_cell(environment: &TableEnvironment, combatant: &CombatantRecord, blur_key: &str, column_styles: &HashMap<String, ColumnStyle>) -> Element {
    let context = use_context::<AppContext>();
    let can_blur = environment.can_blur_names;
    let blur_key = blur_key.to_string();
    let icon = job_icon_view(environment.settings, combatant, true);
    let column_style = column_styles.get("Class").unwrap_or(&NO_COLUMN_STYLE);
    rsx! {
        td {
            key: "{blur_key}",
            class: "Class cell col-cell-body not-last:[border-right:var(--chrome-cell-border)]",
            width: "{column_style.width}",
            padding: "{column_style.padding}",
            text_align: "{column_style.align}",
            cursor: if can_blur { "pointer" },
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
