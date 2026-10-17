//! A member's numbers and names as the domain keeps them: a plain value, with no serde and no ACT field names.
//! The kind of member (player, NPC, companion, Limit Break) is not stored here; the Encounter decides it once and
//! wraps these in the matching type (`member.rs`).
//!
//! (DDD: the wire type stays at the edge, an anticorruption layer; the domain owns its own model.)

use super::encounter::{Amount, Job, Percent, Rate};
use super::{Healing, MemberIdentity};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Figures {
    /// Position in ACT's message (0 = first). Not part of the JSON. Rows that tie on the value a table
    /// is sorted by keep this order, which is what the original's stable sort does.
    pub arrival_index: usize,
    pub name: String,
    pub job: Job,
    /// The member's own duration, e.g. "00:34".
    pub duration_text: String,
    pub duration_seconds: f64,

    pub damage: f64,
    pub damage_per_second: f64,
    pub hits: f64,
    pub misses: f64,
    pub swings: f64,
    pub critical_hits: f64,
    pub direct_hits: f64,
    pub critical_direct_hits: f64,
    pub damage_taken: f64,

    pub heals: f64,
    /// Everything: HP restored + overheal + shields. See `healing()`.
    pub healed: f64,
    pub heal_per_second: f64,
    pub cures: f64,
    pub critical_heals: f64,
    pub heals_taken: f64,
    pub damage_shield: f64,
    pub over_heal: f64,
    pub absorb_heal: f64,

    pub damage_per_second_last_10_seconds: f64,
    pub damage_per_second_last_30_seconds: f64,
    pub damage_per_second_last_60_seconds: f64,
    pub damage_per_second_last_180_seconds: f64,

    /// The action of the strongest hit ("Shieldsplitter"); the amount is
    /// `strongest_hit_amount`. Empty when there was none. (ACT sends "Shieldsplitter-1819".)
    pub strongest_hit_name: String,
    pub strongest_hit_amount: f64,
    /// The action of the strongest heal ("Divine Veil (*)"); the amount is
    /// `strongest_heal_amount`. Empty when there was none.
    pub strongest_heal_name: String,
    pub strongest_heal_amount: f64,


    pub avoided_hits: f64,
    pub mana_restored: f64,
    pub parry_percent: f64,
    pub block_percent: f64,
    pub deaths: f64,

    /// After pets are merged: the part of `damage` that came from pets. 0 otherwise.
    /// For drawing the pet's segment of the DPS bar.
    pub pet_damage: f64,
    /// After pets are merged: the part of the effective healing that came from pets.
    /// For drawing the pet's segment of the HPS bar.
    pub pet_effective_healed: f64,
}

impl Figures {
    /// Damage per second over the member's *own* active time (ACT's `dps`, the overlay's "P.DPS"). A time of 0 counts as
    /// 1 second, as in the original.
    pub fn personal_damage_per_second(&self) -> Rate {
        let seconds = if self.duration_seconds == 0.0 { 1.0 } else { self.duration_seconds };
        Rate::per_second(Amount::new(self.damage), seconds)
    }

    /// The name and job the domain rules work with.
    pub fn identity(&self) -> MemberIdentity<'_> {
        MemberIdentity { name: &self.name, job: self.job.as_str() }
    }

    /// Healing split into what was restored, overhealed and shielded.
    pub fn healing(&self) -> Healing {
        Healing { healed: self.healed, over_heal: self.over_heal, damage_shield: self.damage_shield }
    }

    /// ACT's `critheal%`, rounded like the original; 0 when there were no heals.
    pub fn critical_heal_share(&self) -> Percent {
        Percent::share(Amount::new(self.critical_heals), Amount::new(self.heals))
    }

    pub fn absorb_pet(&mut self, pet: &Figures) {
        self.damage += pet.damage;
        self.damage_per_second += pet.damage_per_second;
        self.hits += pet.hits;
        self.misses += pet.misses;
        self.swings += pet.swings;
        self.critical_hits += pet.critical_hits;
        self.direct_hits += pet.direct_hits;
        self.critical_direct_hits += pet.critical_direct_hits;
        self.damage_taken += pet.damage_taken;

        self.heals += pet.heals;
        self.healed += pet.healed;
        self.heal_per_second += pet.heal_per_second;
        self.cures += pet.cures;
        self.critical_heals += pet.critical_heals;
        self.heals_taken += pet.heals_taken;
        self.damage_shield += pet.damage_shield;
        self.over_heal += pet.over_heal;
        self.absorb_heal += pet.absorb_heal;

        self.damage_per_second_last_10_seconds += pet.damage_per_second_last_10_seconds;
        self.damage_per_second_last_30_seconds += pet.damage_per_second_last_30_seconds;
        self.damage_per_second_last_60_seconds += pet.damage_per_second_last_60_seconds;
        self.damage_per_second_last_180_seconds += pet.damage_per_second_last_180_seconds;

        self.pet_damage += pet.damage;
        self.pet_effective_healed += pet.healing().effective();

        if pet.strongest_hit_amount > self.strongest_hit_amount {
            self.strongest_hit_amount = pet.strongest_hit_amount;
            self.strongest_hit_name.clone_from(&pet.strongest_hit_name);
        }
        if pet.strongest_heal_amount > self.strongest_heal_amount {
            self.strongest_heal_amount = pet.strongest_heal_amount;
            self.strongest_heal_name.clone_from(&pet.strongest_heal_name);
        }
    }

}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/figures.rs"]
mod tests;
