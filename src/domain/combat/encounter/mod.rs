//! The Encounter: one fight as ACT reports it. It is the aggregate root: everything inside is reached through it,
//! and its rules hold whenever it exists (DDD: aggregate, entity, invariants).
//!
//! * `values`  – Amount, Rate, Percent, Job
//! * `events`  – EncounterEnded, YouChanged
//! * `Member`  – the kinds of participant it holds (`super::member`)
//!
//! Rules enforced here:
//! 0. A companion's owner is a player of the same Encounter (or the local player, listed as "YOU"), and there is at
//!    most one Limit Break. `new` refuses anything else; `from_figures` is the door ACT's data comes through and places
//!    the odd ones as `Unclassified` instead, so nothing ACT sent is lost.
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

use super::{Figures, Member};
use super::{plan_pet_merges, MemberKind, EncounterKey, LOCAL_PLAYER};
use std::cmp::Ordering;
use std::collections::HashSet;

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
    facts: EncounterFacts,
    members: Vec<Member>,
    is_active: bool,
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
    /// `members` in the order ACT sent them; each one's `arrival_index` is set here. Refuses a broken Encounter.
    pub fn new(facts: EncounterFacts, mut members: Vec<Member>, is_active: bool) -> Result<Self, EncounterError> {
        check_rules(&members)?;
        for (index, member) in members.iter_mut().enumerate() {
            member.arrival_index = index;
        }
        Ok(Self { facts, members, is_active })
    }

    /// Builds an Encounter from ACT's rows: each is sorted into its kind once, and the ones that break a rule
    /// (a companion without its owner, a second gauge) become `Unclassified`, still shown as they were.
    pub fn from_figures(facts: EncounterFacts, figures: Vec<Figures>, is_active: bool) -> Self {
        let mut members: Vec<Member> = figures.into_iter().map(Member::sort).collect();
        let has_local_player = members.iter().any(|m| matches!(m, Member::Player(_)) && m.name == LOCAL_PLAYER);
        let player_names: HashSet<String> =
            members.iter().filter(|m| matches!(m, Member::Player(_))).map(|m| m.name.clone()).collect();
        let mut seen_limit_break = false;
        for member in members.iter_mut() {
            let must_demote = match member {
                Member::Companion(c) => !(player_names.contains(&c.owner) || has_local_player),
                Member::LimitBreak(_) => std::mem::replace(&mut seen_limit_break, true),
                _ => false,
            };
            if must_demote {
                *member = std::mem::take(member).demoted();
            }
        }
        Self::new(facts, members, is_active).expect("rows that break a rule were placed as Unclassified")
    }

    pub fn facts(&self) -> &EncounterFacts {
        &self.facts
    }

    /// True while the fight is still running.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn players(&self) -> impl Iterator<Item = &Member> {
        self.members.iter().filter(|m| matches!(m, Member::Player(_)))
    }

    pub fn npcs(&self) -> impl Iterator<Item = &Member> {
        self.members.iter().filter(|m| matches!(m, Member::Npc(_)))
    }

    pub fn companions(&self) -> impl Iterator<Item = &Member> {
        self.members.iter().filter(|m| matches!(m, Member::Companion(_)))
    }

    /// The Limit Break gauge, if ACT listed one. There is at most one.
    pub fn limit_break(&self) -> Option<&Member> {
        self.members.iter().find(|m| matches!(m, Member::LimitBreak(_)))
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
        let folded = plan_pet_merges(self.members.iter().map(|m| (m.name.as_str(), m.kind())), None);
        self.members
            .iter()
            .filter(|member| matches!(member.kind(), MemberKind::Pet { .. }) && !folded.iter().any(|merge| merge.pet == member.name))
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

    /// The local player's standing, in the rows as `pet_view` shows them. `None` while the local player is not in the fight.
    pub fn local_standing(&self, pet_view: PetView<'_>) -> Option<Standing> {
        let rows = self.rows(pet_view);
        let damage_index = rows.iter().position(|row| row.name == LOCAL_PLAYER)?;
        let mut by_healing: Vec<&Member> = rows.iter().collect();
        by_healing.sort_by(|a, b| b.healed.partial_cmp(&a.healed).unwrap_or(Ordering::Equal));
        let healing_index = by_healing.iter().position(|row| row.name == LOCAL_PLAYER).unwrap_or(damage_index);
        let player = &rows[damage_index];
        Some(Standing {
            damage_per_second: self.damage_per_second_of(player),
            healing_per_second: self.healing_per_second_of(player),
            damage_rank: damage_index + 1,
            healing_rank: healing_index + 1,
            of: rows.len(),
        })
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

fn check_rules(members: &[Member]) -> Result<(), EncounterError> {
    let player_names: HashSet<&str> = members.iter().filter(|m| matches!(m, Member::Player(_))).map(|m| m.name.as_str()).collect();
    let has_local_player = player_names.contains(LOCAL_PLAYER);
    if let Some(Member::Companion(c)) =
        members.iter().find(|m| matches!(m, Member::Companion(c) if !(player_names.contains(c.owner.as_str()) || has_local_player)))
    {
        return Err(EncounterError::CompanionWithoutOwner { companion: c.figures.name.clone(), owner: c.owner.clone() });
    }
    if members.iter().filter(|m| matches!(m, Member::LimitBreak(_))).count() > 1 {
        return Err(EncounterError::SeveralLimitBreaks);
    }
    Ok(())
}

/// Adds each folded pet's numbers to its owner and removes the pet's row. The plan only names owners that are
/// present; should one be missing anyway, the pet's row is left as it is.
fn fold_pets_into_owners(rows: &mut Vec<Member>, local_player_name: Option<&str>) {
    let merges = plan_pet_merges(rows.iter().map(|m| (m.name.as_str(), m.kind())), local_player_name);
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
