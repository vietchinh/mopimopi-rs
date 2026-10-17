//! The history of finished encounters: its own aggregate, because two rules span several fights.
//!
//! 1. A fight that repeats the previous one (same key: title and totals) is not recorded again.
//! 2. Each fight is numbered within its zone: 1 for the first fight or a new zone, otherwise one more than the
//!    previous fight's number.
//!
//! (DDD: aggregate with invariants that no single Encounter can hold; rules live in the model, not in the signals
//! that happen to store it.) The list is newest first and has no size limit; phase 7 of the plan measures what that costs.

use super::{Encounter, EncounterKey, Member, PetView, LOCAL_PLAYER};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryEntry {
    key: EncounterKey,
    number_in_zone: usize,
    encounter: Rc<Encounter>,
}

impl HistoryEntry {
    pub fn key(&self) -> &EncounterKey {
        &self.key
    }

    /// Which fight in a row this is within the same zone.
    pub fn number_in_zone(&self) -> usize {
        self.number_in_zone
    }

    pub fn encounter(&self) -> &Rc<Encounter> {
        &self.encounter
    }

    /// ACT calls an encounter without a target "Encounter"; the overlay says "No Data".
    pub fn title(&self) -> &str {
        let title = &self.encounter.facts().title;
        if title == "Encounter" { "No Data" } else { title }
    }

    pub fn zone_name(&self) -> &str {
        &self.encounter.facts().zone_name
    }

    pub fn duration_text(&self) -> &str {
        &self.encounter.facts().duration_text
    }

    pub fn encounter_dps(&self) -> f64 {
        self.encounter.facts().damage_per_second
    }

    pub fn encounter_hps(&self) -> f64 {
        self.encounter.facts().heal_per_second
    }

    /// The local player's row as the pet view shows it (their pets added when folded).
    fn local_player_row(&self, pet_view: PetView<'_>) -> Option<Member> {
        self.encounter.rows(pet_view).into_iter().find(|row| row.name == LOCAL_PLAYER)
    }

    pub fn local_player_dps(&self, pet_view: PetView<'_>) -> Option<f64> {
        self.local_player_row(pet_view).map(|player| self.encounter.facts().whole_damage_per_second_of(&player))
    }

    pub fn local_player_hps(&self, pet_view: PetView<'_>) -> Option<f64> {
        self.local_player_row(pet_view).map(|player| self.encounter.facts().whole_healing_per_second_of(&player))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct History {
    /// Newest first.
    entries: Vec<HistoryEntry>,
}

impl History {
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn find(&self, key: &EncounterKey) -> Option<&HistoryEntry> {
        self.entries.iter().find(|entry| entry.key() == key)
    }

    /// Whether this encounter repeats the previous one (rule 1), so recording it would change nothing.
    pub fn is_repeat(&self, encounter: &Encounter) -> bool {
        self.entries.first().is_some_and(|entry| entry.key == encounter.key())
    }

    /// Records a finished encounter. `None` (nothing changes) when it repeats the previous one (rule 1); otherwise the
    /// new entry, numbered within its zone (rule 2).
    pub fn record(&mut self, encounter: Rc<Encounter>) -> Option<&HistoryEntry> {
        if self.is_repeat(&encounter) {
            return None;
        }
        let key = encounter.key();
        let previous = self.entries.first();
        let number_in_zone = match previous {
            Some(previous) if previous.zone_name() == encounter.facts().zone_name => previous.number_in_zone + 1,
            _ => 1,
        };
        self.entries.insert(0, HistoryEntry { key, number_in_zone, encounter });
        self.entries.first()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/history.rs"]
mod tests;
