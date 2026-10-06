//! The columns of the damage and healing tables: which are shown, in which order, and how each is laid out.

use crate::domain::settings::{Align, SettingsFile};
use crate::presentation::ui::shared::style::Size;
use std::fmt::{self, Display};

/// A column's width: a size, or the rest of the row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnWidth {
    Fill,
    Fixed(Size),
}

impl Display for ColumnWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColumnWidth::Fill => f.write_str("100%"),
            ColumnWidth::Fixed(size) => size.fmt(f),
        }
    }
}

/// One column of a table, as every cell of it is laid out.
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    /// The column's name: its key in the settings, its class, and what the cells show.
    pub name: String,
    pub title: String,
    pub width: ColumnWidth,
    pub padding: Size,
    pub header_align: Align,
    pub body_align: Align,
}

impl Column {
    /// `padding` as the value of the attribute: nothing above or below, the padding on both sides.
    pub fn padding_css(&self) -> String {
        format!("0 {}", self.padding)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnSettings {
    damage: Vec<Column>,
    healing: Vec<Column>,
}

impl ColumnSettings {
    /// The only place that knows how the columns are stored: `Order` lists a table's columns, `ColData` defines each, and a column whose
    /// switch for the table is off is not shown (`SettingsFile::enabled_columns`; the header and the body take their columns from it).
    pub fn from_raw(raw: &SettingsFile) -> Self {
        let columns_of = |table: &str| -> Vec<Column> {
            raw.enabled_columns(table)
                .into_iter()
                .filter_map(|name| raw.columns.get(name).map(|definition| (name, definition)))
                .map(|(name, definition)| Column {
                    name: name.to_string(),
                    title: definition.tt.clone(),
                    // The name column takes the rest of the row, whatever width its definition holds.
                    width: if name == "name" { ColumnWidth::Fill } else { ColumnWidth::Fixed(Size(definition.width.as_f64())) },
                    padding: Size(definition.padding.as_f64()),
                    header_align: definition.align_header,
                    body_align: definition.align_body,
                })
                .collect()
        };
        ColumnSettings { damage: columns_of("DPS"), healing: columns_of("HPS") }
    }

    /// The columns of the damage or the healing table, in the order they are drawn.
    pub fn of(&self, is_healing: bool) -> &[Column] {
        if is_healing { &self.healing } else { &self.damage }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/areas/columns.rs"]
mod tests;
