//! The Encounter: one fight as ACT reports it. It is the aggregate root: everything inside is reached through it,
//! and its rules hold whenever it exists (DDD: aggregate, entity, invariants).
//!
//! * `values`  – Amount, Rate, Percent, Job
//! * `events`  – EncounterEnded, YouChanged
//! * `Member`  – one participant (`super::member`)
//!
//! Rules enforced here:
//! 1. The Encounter is never changed by display choices. Folding pets into their owners is a *view* (`rows` with
//!    `PetView::FoldedIntoOwners`), a side-effect-free question the application asks when the `pets` option says so.
//!    Nothing is merged when a message arrives.
//! 2. Rows are always ordered by damage, ties in the order ACT sent them (the original always sorts, and ACT's own
//!    order is arbitrary).
//! 3. A pet whose owner is not in the fight stays as its own row (`orphaned_pets`), so nothing ACT sent is lost.
//! 4. Rates come from the Encounter's duration (`damage_per_second_of`, ACT's `encdps`) or from a member's own
//!    active time (`Member::personal_damage_per_second`, ACT's `dps`).

mod events;
mod values;

pub use events::{EncounterEnded, YouChanged};
pub use values::{Amount, Job, Percent, Rate};

use super::member::Member;
use super::{plan_pet_merges, CombatantKind, EncounterKey, LOCAL_PLAYER};
use std::cmp::Ordering;

/// What ACT says about the fight as a whole.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EncounterFacts {
    pub title: String,
    pub duration_text: String,
    pub duration_seconds: f64,
    pub total_damage: f64,
    pub total_healed: f64,
    /// ACT's `encdps` ("6106.61"), not the rounded `ENCDPS` ("6107").
    pub damage_per_second: f64,
    pub heal_per_second: f64,
    /// ACT's whole-number `ENCDPS`, which the top bar's "Total DPS" prints as it is.
    pub damage_per_second_whole: f64,
    pub heal_per_second_whole: f64,
    pub zone_name: String,
}

/// How the rows of an Encounter are shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetView<'a> {
    /// Every pet is its own row.
    Separate,
    /// Pets are added to their owners. `local_player_name` lets the player's own pets find the "YOU" row.
    FoldedIntoOwners { local_player_name: Option<&'a str> },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Encounter {
    facts: EncounterFacts,
    members: Vec<Member>,
    is_active: bool,
}

impl Encounter {
    /// `members` in the order ACT sent them; each one's `arrival_index` is set here.
    pub fn new(facts: EncounterFacts, mut members: Vec<Member>, is_active: bool) -> Self {
        for (index, member) in members.iter_mut().enumerate() {
            member.arrival_index = index;
        }
        Self { facts, members, is_active }
    }

    pub fn facts(&self) -> &EncounterFacts {
        &self.facts
    }

    /// True while the fight is still running.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// The members as ACT sent them, never folded.
    pub fn members(&self) -> &[Member] {
        &self.members
    }

    /// Identifies this fight (title and totals); the history uses it to skip a repeat.
    pub fn key(&self) -> EncounterKey {
        EncounterKey::new(&self.facts.title, self.facts.total_damage, self.facts.total_healed)
    }

    /// The fact that this message ended the fight, if the previous message was from a running one.
    pub fn ended_since(&self, previous_message_was_active: bool) -> Option<EncounterEnded> {
        (previous_message_was_active && !self.is_active).then(|| EncounterEnded { key: self.key() })
    }

    /// The local player's row, if they are in this fight. ACT always lists the local player as "YOU".
    pub fn local_player(&self) -> Option<&Member> {
        self.members.iter().find(|member| member.name == LOCAL_PLAYER)
    }

    /// Pets whose owner is not in this fight (they keep their own row whatever the `PetView`).
    pub fn orphaned_pets(&self) -> Vec<&Member> {
        let folded = plan_pet_merges(self.members.iter().map(Member::identity), None);
        self.members
            .iter()
            .filter(|member| matches!(member.kind(), CombatantKind::Pet { .. }) && !folded.iter().any(|merge| merge.pet == member.name))
            .collect()
    }

    /// The rows to show, most damage first. A question, not a command: the Encounter itself does not change.
    pub fn rows(&self, pet_view: PetView<'_>) -> Vec<Member> {
        let mut rows = self.members.clone();
        if let PetView::FoldedIntoOwners { local_player_name } = pet_view {
            fold_pets_into_owners(&mut rows, local_player_name);
        }
        rows.sort_by(|a, b| b.damage.partial_cmp(&a.damage).unwrap_or(Ordering::Equal).then(a.arrival_index.cmp(&b.arrival_index)));
        rows
    }

    /// See `EncounterFacts::damage_per_second_of`.
    pub fn damage_per_second_of(&self, member: &Member) -> Rate {
        self.facts.damage_per_second_of(member)
    }

    pub fn healing_per_second_of(&self, member: &Member) -> Rate {
        self.facts.healing_per_second_of(member)
    }
}

impl EncounterFacts {
    /// Damage per second over the *Encounter's* duration (ACT's `encdps`, the overlay's "DPS"). A duration of 0 gives
    /// infinity, as in JavaScript. It is recomputed from the damage on every message, never read from ACT's own rate
    /// field: that one goes stale while a player lands no hits, but the Encounter's duration keeps ticking, so the
    /// recomputed rate visibly falls, like in the original.
    pub fn damage_per_second_of(&self, member: &Member) -> Rate {
        Rate::per_second(Amount::new(member.damage), self.duration_seconds)
    }

    pub fn healing_per_second_of(&self, member: &Member) -> Rate {
        Rate::per_second(Amount::new(member.healed), self.duration_seconds)
    }

    /// The history screen's "My DPS": the same division, cut off to whole numbers instead of rounded.
    pub fn whole_damage_per_second_of(&self, member: &Member) -> f64 {
        (member.damage / self.duration_seconds).floor()
    }

    pub fn whole_healing_per_second_of(&self, member: &Member) -> f64 {
        (member.healed / self.duration_seconds).floor()
    }
}

/// Adds each folded pet's numbers to its owner and removes the pet's row. The plan only names owners that are
/// present; should one be missing anyway, the pet's row is left as it is.
fn fold_pets_into_owners(rows: &mut Vec<Member>, local_player_name: Option<&str>) {
    let merges = plan_pet_merges(rows.iter().map(Member::identity), local_player_name);
    for merge in &merges {
        let Some(owner_index) = rows.iter().position(|member| member.name == merge.owner) else { continue };
        let Some(pet_index) = rows.iter().position(|member| member.name == merge.pet) else { continue };
        let pet = rows.remove(pet_index);
        let owner_index = if pet_index < owner_index { owner_index - 1 } else { owner_index };
        rows[owner_index].absorb_pet(&pet);
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/domain/combat/encounter.rs"]
mod tests;
