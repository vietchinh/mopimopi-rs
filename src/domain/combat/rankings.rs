//! Builds the two rankings (by damage and by healing) the tables need from one ACT message.

/// Which of the two tables is meant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableKind {
    Damage,
    Healing,
}

impl TableKind {
    /// "DPS" or "HPS": the prefix used in element ids, column flags and setting keys.
    pub fn short_label(self) -> &'static str {
        match self {
            TableKind::Damage => "DPS",
            TableKind::Healing => "HPS",
        }
    }
}
