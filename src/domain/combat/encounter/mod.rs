//! The Encounter: one fight as ACT reports it. It is the aggregate root: everything inside is reached through it,
//! and its rules hold whenever it exists (DDD: aggregate, entity, invariants).
//!
//! * `values`  – Amount, Rate, Percent, Job
//! * `events`  – EncounterEnded, YouChanged
//! * `phase`   – Running or Over: the life of a fight
//! * `Participant`  – the kinds of participant it holds (`super::participant`)
//!
//! Rules enforced here:
//! 0. A companion's owner is a player of the same Encounter (or the local player, listed as "YOU"), and there is at
//!    most one Limit Break. `new` refuses anything else; `from_participants` is the door the edge's data comes through and places
//!    the odd ones as `Unclassified` instead, so nothing ACT sent is lost.
//! 1. The Encounter is never changed by display choices. Folding pets into their owners is a *view* (`rows` with
//!    `PetView::FoldedIntoOwners`), a side-effect-free question the application asks when the `pets` option says so.
//!    Nothing is merged when a message arrives.
//! 2. Rows are always ordered by damage, ties in the order ACT sent them (the original always sorts, and ACT's own
//!    order is arbitrary).
//! 3. A pet whose owner is not in the fight stays as its own row whatever the `PetView`, so nothing ACT sent is lost.
//! 4. Rates come from the Encounter's duration (`damage_per_second_of`, ACT's `encdps`) or from a participant's own
//!    active time (`Participant::personal_damage_per_second`, ACT's `dps`).

mod events;
mod phase;
mod values;

pub use events::{EncounterEnded, YouChanged};
pub use phase::Phase;
pub use values::{Amount, Job, Percent, Rate, Reported};

use super::Participant;
use super::{plan_pet_merges, EncounterKey, PetCandidate};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;

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

/// Where the local player stands in a fight: the top bar's "My DPS", "My HPS" and "Rank". Computed on demand from the
/// Encounter, never stored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Standing {
    pub damage_per_second: Rate,
    pub healing_per_second: Rate,
    /// 1-based, by damage.
    pub damage_rank: usize,
    /// 1-based, by healing.
    pub healing_rank: usize,
    pub of: usize,
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
    /// Shared (`Rc`), like the participants: whoever draws a row takes another handle to the same data, not a copy.
    facts: Rc<EncounterFacts>,
    participants: Vec<Rc<Participant>>,
    phase: Phase,
}

/// Why an Encounter could not be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncounterError {
    /// A companion's owner is neither a player of the fight nor the local player.
    CompanionWithoutOwner { companion: String, owner: String },
    /// More than one Limit Break gauge.
    SeveralLimitBreaks,
}

impl Encounter {
    /// The strict door: `participants` in the order ACT sent them, refused if any rule is broken. The app comes in through
    /// `from_participants` (which places the rows that break a rule instead of refusing the fight), so only the tests use this one.
    #[cfg(test)]
    pub fn new(facts: EncounterFacts, participants: Vec<Participant>, phase: Phase) -> Result<Self, EncounterError> {
        check_rules(&participants)?;
        Ok(Self::assemble(facts, participants, phase))
    }

    /// Builds an Encounter from participants already placed by the sender's edge. The ones that break a rule
    /// (a companion without its owner, a second gauge) become `Unclassified`, still shown as they were.
    pub fn from_participants(facts: EncounterFacts, mut participants: Vec<Participant>, phase: Phase) -> Self {
        for index in indices_to_demote(&participants) {
            participants[index] = std::mem::take(&mut participants[index]).demoted();
        }
        debug_assert!(check_rules(&participants).is_ok(), "rows that break a rule were placed as Unclassified");
        Self::assemble(facts, participants, phase)
    }

    /// Numbers the participants in arrival order and shares them. The rules were checked (or made true) by the caller.
    fn assemble(facts: EncounterFacts, mut participants: Vec<Participant>, phase: Phase) -> Self {
        for (index, participant) in participants.iter_mut().enumerate() {
            participant.arrival_index = index;
        }
        Self { facts: Rc::new(facts), participants: participants.into_iter().map(Rc::new).collect(), phase }
    }

    pub fn facts(&self) -> &EncounterFacts {
        &self.facts
    }

    /// The same facts as a shared handle, for whoever has to keep them (a component's props) without copying.
    pub fn shared_facts(&self) -> Rc<EncounterFacts> {
        Rc::clone(&self.facts)
    }

    /// Running while the fight goes on, Over after it.
    pub fn is_running(&self) -> bool {
        self.phase.is_running()
    }

    /// The participants as ACT sent them, never folded. Only tests read this order; the app asks `rows`.
    #[cfg(test)]
    pub fn participants(&self) -> &[Rc<Participant>] {
        &self.participants
    }

    /// Identifies this fight (title and totals, as the original does: ACT sends no id). Meant for a fight that is Over;
    /// the history uses it to skip a repeat.
    pub fn key(&self) -> EncounterKey {
        EncounterKey::new(&self.facts.title, self.facts.total_damage, self.facts.total_healed)
    }

    /// The fact that this message ended the fight, if the previous message was from a running one.
    pub fn ended_since(&self, previous: Phase) -> Option<EncounterEnded> {
        (previous.is_running() && !self.is_running()).then(|| EncounterEnded { key: self.key() })
    }

    /// The rows to show, most damage first. A question, not a command: the Encounter itself does not change.
    /// The rows are shared with the Encounter, not copied: each is another handle (`Rc`) to the participant. Only an
    /// owner that takes in a pet's numbers is copied (and only in the folded view), so asking for the rows on every
    /// redraw copies next to nothing.
    pub fn rows(&self, pet_view: PetView<'_>) -> Vec<Rc<Participant>> {
        let mut rows: Vec<Rc<Participant>> = self.participants.iter().map(Rc::clone).collect();
        if let PetView::FoldedIntoOwners { local_player_name } = pet_view {
            fold_pets_into_owners(&mut rows, local_player_name);
        }
        rows.sort_by(|a, b| b.damage.partial_cmp(&a.damage).unwrap_or(Ordering::Equal).then(a.arrival_index.cmp(&b.arrival_index)));
        rows
    }

    /// The local player's standing, in the rows as `pet_view` shows them. `None` while the local player is not in the fight.
    pub fn local_standing(&self, pet_view: PetView<'_>) -> Option<Standing> {
        let rows = self.rows(pet_view);
        let damage_index = rows.iter().position(|row| matches!(row.as_ref(), Participant::You(_)))?;
        let mut by_healing: Vec<&Participant> = rows.iter().map(|row| row.as_ref()).collect();
        by_healing.sort_by(|a, b| b.healed.partial_cmp(&a.healed).unwrap_or(Ordering::Equal));
        let healing_index = by_healing.iter().position(|row| matches!(row, Participant::You(_))).unwrap_or(damage_index);
        let you = &rows[damage_index];
        Some(Standing {
            damage_per_second: self.damage_per_second_of(you),
            healing_per_second: self.healing_per_second_of(you),
            damage_rank: damage_index + 1,
            healing_rank: healing_index + 1,
            of: rows.len(),
        })
    }

    /// See `EncounterFacts::damage_per_second_of`.
    pub fn damage_per_second_of(&self, participant: &Participant) -> Rate {
        self.facts.damage_per_second_of(participant)
    }

    pub fn healing_per_second_of(&self, participant: &Participant) -> Rate {
        self.facts.healing_per_second_of(participant)
    }
}

impl EncounterFacts {
    /// Damage per second over the *Encounter's* duration (ACT's `encdps`, the overlay's "DPS"). A duration of 0 gives
    /// infinity, as in JavaScript. It is recomputed from the damage on every message, never read from ACT's own rate
    /// field: that one goes stale while a player lands no hits, but the Encounter's duration keeps ticking, so the
    /// recomputed rate visibly falls, like in the original.
    pub fn damage_per_second_of(&self, participant: &Participant) -> Rate {
        Rate::per_second(participant.damage, self.duration_seconds)
    }

    pub fn healing_per_second_of(&self, participant: &Participant) -> Rate {
        Rate::per_second(participant.healed, self.duration_seconds)
    }

    /// The history screen's "My DPS": the same division, cut off to whole numbers instead of rounded.
    pub fn whole_damage_per_second_of(&self, participant: &Participant) -> f64 {
        (participant.damage.value() / self.duration_seconds).floor()
    }

    pub fn whole_healing_per_second_of(&self, participant: &Participant) -> f64 {
        (participant.healed.value() / self.duration_seconds).floor()
    }
}

fn check_rules(participants: &[Participant]) -> Result<(), EncounterError> {
    let player_names: HashSet<&str> = participants.iter().filter(|participant| matches!(participant, Participant::Player(_) | Participant::You(_))).map(|participant| participant.name.as_str()).collect();
    let has_you = participants.iter().any(|participant| matches!(participant, Participant::You(_)));
    if let Some(Participant::Companion(companion)) =
        participants.iter().find(|participant| matches!(participant, Participant::Companion(companion) if !(player_names.contains(companion.owner.as_str()) || has_you)))
    {
        return Err(EncounterError::CompanionWithoutOwner { companion: companion.stats.name.clone(), owner: companion.owner.clone() });
    }
    if participants.iter().filter(|participant| matches!(participant, Participant::LimitBreak(_))).count() > 1 {
        return Err(EncounterError::SeveralLimitBreaks);
    }
    Ok(())
}

fn pet_candidate(participant: &Participant) -> PetCandidate<'_> {
    PetCandidate { name: &participant.name, pet_owner: participant.pet_owner(), is_you: participant.is_you() }
}

/// The positions of the rows that break a rule: a companion whose owner is not in the fight, and every Limit Break after the
/// first. Allocates nothing in the usual fight: a companion's owner is only looked up when the local player ("YOU", who owns
/// every unowned companion) is absent, and an empty `Vec` has no heap.
fn indices_to_demote(participants: &[Participant]) -> Vec<usize> {
    let has_you = participants.iter().any(|participant| matches!(participant, Participant::You(_)));
    let has_companion = participants.iter().any(|participant| matches!(participant, Participant::Companion(_)));
    let players_by_name: Option<HashSet<&str>> = (!has_you && has_companion)
        .then(|| participants.iter().filter(|participant| matches!(participant, Participant::Player(_) | Participant::You(_))).map(|participant| participant.name.as_str()).collect());
    let ownerless_companions = participants.iter().enumerate().filter(|(_, participant)| match participant {
        Participant::Companion(companion) => players_by_name.as_ref().is_some_and(|names| !names.contains(companion.owner.as_str())),
        _ => false,
    });
    let extra_limit_breaks = participants.iter().enumerate().filter(|(_, participant)| matches!(participant, Participant::LimitBreak(_))).skip(1);
    ownerless_companions.chain(extra_limit_breaks).map(|(index, _)| index).collect()
}

/// Adds each folded pet's numbers to its owner and removes the pet's row. The plan only names owners that are
/// present; should one be missing anyway, the pet's row is left as it is.
fn fold_pets_into_owners(rows: &mut Vec<Rc<Participant>>, local_player_name: Option<&str>) {
    let merges = plan_pet_merges(rows.iter().map(|row| pet_candidate(row.as_ref())), local_player_name);
    for merge in &merges {
        let Some(owner_index) = rows.iter().position(|participant| participant.name == merge.owner) else { continue };
        let Some(pet_index) = rows.iter().position(|participant| participant.name == merge.pet) else { continue };
        let pet = rows.remove(pet_index);
        let owner_index = if pet_index < owner_index { owner_index - 1 } else { owner_index };
        Rc::make_mut(&mut rows[owner_index]).absorb_pet(&pet);
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/domain/combat/encounter.rs"]
mod tests;
