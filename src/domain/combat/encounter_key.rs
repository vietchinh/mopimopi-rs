//! Identifies one finished encounter (value object).
//!
//! Two encounters with the same title and the same totals are "the same fight": the history uses this to
//! skip a repeat, and the History screen uses it as the row id. Kept as a type, not a bare `String`, so a
//! key cannot be confused with any other text (talk rule 5: preconditions in types).

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EncounterKey(String);

impl EncounterKey {
    /// Same text as the original overlay's key: title, damage total and healing total joined without separator.
    pub fn new(title: &str, total_damage: f64, total_healed: f64) -> Self {
        Self(format!("{title}{total_damage}{total_healed}"))
    }

    #[allow(dead_code)] // for tests and for showing the key as text
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EncounterKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/encounter_key.rs"]
mod tests;
