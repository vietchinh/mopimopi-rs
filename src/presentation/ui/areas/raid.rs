//! Raid mode: the grid of small cards, one per player.

use crate::domain::settings::SettingsFile;
use crate::presentation::ui::shared::style::{Line, Opacity, Paint, Size, StyleReads, TextStyle, Vars};

#[derive(Clone, Debug, PartialEq)]
pub struct RaidSettings {
    /// The line between cards, the same as the tables' row line.
    pub line: Line,
    /// How many cards fit in a row, as the setting has it (at least 1).
    cards_per_row: f64,
    pub background_other: Paint,
    pub background_own: Paint,
    pub card_height: Size,
    /// The text of the cards of the others and of the local player. Both set everything; the size is each cell's own (name, data).
    pub other: TextStyle,
    pub own: TextStyle,
    pub name_size: Size,
    pub data_size: Size,
    /// The colour strip's width and opacity (the opacity is the bars' own setting).
    pub strip_width: Size,
    pub strip_opacity: Opacity,
    pub icon_width: Size,
}

impl RaidSettings {
    /// The only place that knows the raid grid's settings keys.
    pub fn from_raw(raw: &SettingsFile) -> Self {
        let font = raw.font("fBody", "'Segoe UI', 'sans-serif'");
        let italic = raw.flag("body_italic");
        let text = |color_key: &str, bold_key: &str, border_key: &str| TextStyle {
            color: raw.solid(color_key),
            opacity: raw.opacity(color_key),
            ex_color: None,
            font: font.clone(),
            size: None,
            italic,
            bold: raw.flag(bold_key),
            shadow: raw.shadow(border_key),
        };
        RaidSettings {
            line: raw.solid_line("sizeLine", "tableLine"),
            cards_per_row: raw.size("size24TableSlice").0.max(1.0),
            background_other: raw.paint("view24BgOther"),
            background_own: raw.paint("view24BgYOU"),
            card_height: raw.size("size24TableHeight"),
            other: text("view24TableOther", "boldOther", "tableBorderOther"),
            own: text("view24TableYOU", "boldYOU", "tableBorderYOU"),
            name_size: raw.size("size24BodyNameText"),
            data_size: raw.size("size24BodyDataText"),
            strip_width: raw.size("size24TableIdxWd"),
            strip_opacity: raw.opacity("bar"),
            icon_width: raw.size("size24BodyIcon"),
        }
    }

    /// The number of cards in a row.
    pub fn cards_in_a_row(&self) -> usize {
        self.cards_per_row as usize
    }

    /// Every variable shared by every card of the grid, set once on its container.
    pub fn grid_vars(&self) -> String {
        let mut vars = Vars::default();
        vars.set("raid-cell-border", &self.line)
            .set("raid-cards-per-row", self.cards_per_row)
            .set("raid-cell-bg-other", &self.background_other)
            .set("raid-cell-bg-own", &self.background_own)
            .set("raid-cell-height", self.card_height);
        self.other.write(&mut vars, "text");
        self.own.write(&mut vars, "own");
        vars.set("raid-name-size", self.name_size)
            .set("raid-data-size", self.data_size)
            .set("raid-idx-width", self.strip_width)
            .set("raid-idx-opacity", self.strip_opacity)
            .set("raid-icon-width", self.icon_width);
        vars.finish()
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/areas/raid.rs"]
mod tests;
