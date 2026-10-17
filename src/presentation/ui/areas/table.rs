//! The damage and healing tables: which are shown and in which order, how many rows, which jobs, and the style of the header and the body.

use crate::domain::settings::SettingsFile;
use crate::presentation::ui::shared::style::{Corners, Line, Opacity, Paint, Shadow, Size, StyleReads, TextStyle, Vars};

/// "Apply to" of the corner radius: the header, the body, or both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CornerCoverage {
    HeaderOnly,
    BodyOnly,
    HeaderAndBody,
}

impl CornerCoverage {
    fn from_option(number: i32) -> Self {
        match number {
            1 => CornerCoverage::HeaderOnly,
            2 => CornerCoverage::BodyOnly,
            _ => CornerCoverage::HeaderAndBody,
        }
    }

    fn header(self) -> bool {
        matches!(self, CornerCoverage::HeaderOnly | CornerCoverage::HeaderAndBody)
    }

    fn body(self) -> bool {
        matches!(self, CornerCoverage::BodyOnly | CornerCoverage::HeaderAndBody)
    }
}

/// Which jobs a table shows ("DPS_T" = tanks in the damage table, "HPS_H" = healers in the healing table, ...).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobFilter {
    pub tanks: bool,
    pub healers: bool,
    pub damage_dealers: bool,
    pub chocobos: bool,
    pub crafters_and_gatherers: bool,
}

/// What belongs to one of the two tables.
#[derive(Clone, Debug, PartialEq)]
pub struct TableKind {
    pub shown: bool,
    /// The space above the table's header.
    pub gap: Size,
    /// The number of rows the table shows at most.
    pub row_limit: f64,
    pub filter: JobFilter,
}

/// A graph bar's size and opacity; its margin is what is left of the row's height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarSize {
    pub height: Size,
    pub opacity: Opacity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderStyle {
    pub corners: Corners,
    pub height: Size,
    pub margin: Size,
    pub background: Paint,
    /// The text of the header cells: no opacity, border or dimmed part.
    pub text: TextStyle,
    /// The line between header cells, in the header's own colour.
    pub cell_line: Line,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BodyStyle {
    pub corners: Corners,
    pub row_height: Size,
    pub icon_size: Size,
    /// The size of the dimmed text: one tenth of a rem less than the text.
    pub ex_size: Size,
    pub bar: BarSize,
    pub pet_bar: BarSize,
    pub shield_bar: BarSize,
    pub overheal_bar: BarSize,
    pub bar_corners: Corners,
    pub row_line: Line,
    pub bar_background: Paint,
    pub cell_line: Line,
    /// The text of the local player's rows (you and your pet), and of everyone else's. Both set everything, so the CSS only swaps what differs.
    pub own: TextStyle,
    pub other: TextStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableSettings {
    /// The folder of the job icons (`iconSet`).
    pub icon_set: String,
    /// "Order of tables": the damage table first.
    pub damage_first: bool,
    /// Raid mode (cards instead of rows) is on, from this many players.
    pub raid_enabled: bool,
    pub raid_players: f64,
    pub damage: TableKind,
    pub healing: TableKind,
    pub header: HeaderStyle,
    pub body: BodyStyle,
}

impl TableSettings {
    /// The only place that knows the tables' settings keys.
    pub fn from_raw(raw: &SettingsFile) -> Self {
        let scope = CornerCoverage::from_option(raw.choice::<i32>("applyScope"));
        let corners = |covered: bool| if covered { raw.corners("rd_table", "sizeRadiusTable") } else { Corners::NONE };
        let bar = |height_key: &str, opacity_key: &str| BarSize { height: raw.size(height_key), opacity: raw.opacity(opacity_key) };
        let body_font = raw.font("fBody", "'Segoe UI', 'sans-serif'");
        let body_text_size = raw.size("sizeBodyText");
        let body_italic = raw.flag("body_italic");
        let text = |color_key: &str, ex_key: &str, bold_key: &str, border_key: &str| TextStyle {
            color: raw.solid(color_key),
            opacity: raw.opacity(color_key),
            ex_color: Some(raw.solid(ex_key)),
            font: body_font.clone(),
            size: Some(body_text_size),
            italic: body_italic,
            bold: raw.flag(bold_key),
            shadow: raw.shadow(border_key),
        };
        let header_background = raw.paint("tableHd");
        TableSettings {
            icon_set: raw.text("iconSet"),
            damage_first: raw.choice::<i32>("tableOrder") == 1,
            raid_enabled: raw.flag("view24"),
            raid_players: raw.number("view24_Number"),
            damage: TableKind {
                shown: raw.flag("viewDPS"),
                gap: raw.size("sizeDPSGap"),
                row_limit: raw.size("sizeDPSTable").0.max(0.0),
                filter: JobFilter { tanks: raw.flag("DPS_T"), healers: raw.flag("DPS_H"), damage_dealers: raw.flag("DPS_D"), chocobos: raw.flag("DPS_C"), crafters_and_gatherers: raw.flag("DPS_M") },
            },
            healing: TableKind {
                shown: raw.flag("viewHPS"),
                gap: raw.size("sizeHPSGap"),
                row_limit: raw.size("sizeHPSTable").0.max(0.0),
                filter: JobFilter { tanks: raw.flag("HPS_T"), healers: raw.flag("HPS_H"), damage_dealers: raw.flag("HPS_D"), chocobos: raw.flag("HPS_C"), crafters_and_gatherers: raw.flag("HPS_M") },
            },
            header: HeaderStyle {
                corners: corners(scope.header()),
                height: raw.size("sizeHd"),
                margin: raw.size("sizeHdGap"),
                background: header_background.clone(),
                text: TextStyle {
                    color: raw.paint("tableHdText"),
                    opacity: Opacity::FULL,
                    ex_color: None,
                    font: raw.font("fHd", "'Roboto Condensed', 'sans-serif'"),
                    size: Some(raw.size("sizeHdText")),
                    italic: raw.flag("header_italic"),
                    bold: false,
                    shadow: Shadow::None,
                },
                cell_line: Line { width: raw.size("sizeLineVer"), style: "solid".to_string(), paint: header_background },
            },
            body: BodyStyle {
                corners: corners(scope.body()),
                row_height: raw.size("sizeBody"),
                icon_size: raw.size("sizeBodyIcon"),
                ex_size: Size(body_text_size.0 - 1.0),
                bar: bar("sizeGraph_bar", "bar"),
                pet_bar: bar("sizeGraph_pet", "pet"),
                shield_bar: bar("sizeGraph_ds", "ds"),
                overheal_bar: bar("sizeGraph_oh", "oh"),
                bar_corners: raw.corners("rd_graph", "sizeRadiusGraph"),
                row_line: raw.solid_line("sizeLine", "tableLine"),
                bar_background: raw.paint("tableBg"),
                cell_line: raw.solid_line("sizeLineVer", "tableLineVer"),
                own: text("tableYOU", "tableExYOU", "boldYOU", "tableBorderYOU"),
                other: text("tableOther", "tableExOther", "boldOther", "tableBorderOther"),
            },
        }
    }

    /// The damage or the healing table's own settings.
    pub fn kind(&self, is_healing: bool) -> &TableKind {
        if is_healing { &self.healing } else { &self.damage }
    }

    /// Raid mode for a fight with this many players.
    pub fn raid_mode(&self, player_count: usize) -> bool {
        self.raid_enabled && player_count as f64 >= self.raid_players
    }

    /// The tables to draw, in order: `true` is the healing table.
    pub fn tables_in_order(&self) -> Vec<bool> {
        let order = if self.damage_first { [false, true] } else { [true, false] };
        order.into_iter().filter(|&is_healing| self.kind(is_healing).shown).collect()
    }

    /// Height in rem of a table's body: rows are as tall as the row plus its line, up to the row limit.
    pub fn body_height_rem(&self, is_healing: bool, row_count: usize) -> f64 {
        let visible_rows = (row_count as f64).min(self.kind(is_healing).row_limit);
        visible_rows * (self.body.row_height.0 + self.body.row_line.width.0) / 10.0
    }

    /// Every variable shared by the header cells of a table, for the wrapper (every cell inherits them); the history list has the same header.
    pub fn header_vars(&self) -> String {
        let header = &self.header;
        let mut vars = Vars::default();
        header.corners.write(&mut vars, "table-radius-header");
        vars.set("chrome-header-height", header.height)
            .set("chrome-header-margin", header.margin)
            .set("chrome-header-bg", &header.background)
            .set("chrome-header-cell-border", &header.cell_line);
        header.text.write(&mut vars, "text");
        vars.finish()
    }

    /// Every variable shared by the rows of a table's body: the row and its bars, and the text of own and other rows (both, each row picks).
    pub fn body_vars(&self) -> String {
        let body = &self.body;
        let mut vars = Vars::default();
        body.corners.write(&mut vars, "table-radius-body");
        vars.set("chrome-row-height", body.row_height).set("chrome-icon-size", body.icon_size).set("chrome-ex-size", body.ex_size);
        self.write_bar(&mut vars, "main", body.bar);
        self.write_bar(&mut vars, "pet", body.pet_bar);
        self.write_bar(&mut vars, "ds", body.shield_bar);
        self.write_bar(&mut vars, "oh", body.overheal_bar);
        vars.set("chrome-bar-radius", body.bar_corners.border_radius())
            .set("chrome-row-border", &body.row_line)
            .set("chrome-bar-bg", &body.bar_background)
            .set("chrome-cell-border", &body.cell_line);
        body.other.write(&mut vars, "text");
        body.own.write(&mut vars, "own");
        vars.finish()
    }

    /// `--chrome-bar-{kind}-height`, `-margin` (what is left of the row's height) and `-opacity`.
    fn write_bar(&self, vars: &mut Vars, kind: &str, bar: BarSize) {
        vars.set(&format!("chrome-bar-{kind}-height"), bar.height)
            .set(&format!("chrome-bar-{kind}-margin"), Size(self.body.row_height.0 - bar.height.0))
            .set(&format!("chrome-bar-{kind}-opacity"), bar.opacity);
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/areas/table.rs"]
mod tests;
