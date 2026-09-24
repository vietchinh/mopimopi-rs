//! Turning player data into the text shown in table cells.
//!
//! * `text_fragment`   – text plus dimmed unit text ("12.3" + "k")
//! * `number_format`   – digit grouping, decimal marks, k/M units, from the user's settings
//! * `player_name`     – name abbreviation, owner names, rank prefix
//! * `strongest_action_text` – the MaxHit / MaxHeal cell
//! * `column_cell`     – one function that produces every column's cell

mod column_cell;
mod number_format;
mod player_name;
mod strongest_action_text;
mod text_fragment;

pub use column_cell::{cell_fragments, cell_plain_text, CellContext};
pub use number_format::NumberFormat;
pub use text_fragment::TextFragment;
