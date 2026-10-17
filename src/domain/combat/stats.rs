//! A participant's numbers and names as the domain keeps them: a plain value, with no serde and no ACT field names.
//! The kind of participant (player, NPC, companion, Limit Break) is not stored here; the Encounter decides it once and
//! wraps these in the matching type (`participant.rs`).
//!
//! (DDD: the wire type stays at the edge, an anticorruption layer; the domain owns its own model.)

use super::encounter::{Amount, Job, Percent, Rate, Reported};
use super::Healing;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stats {
    /// Position in ACT's message (0 = first). Not part of the JSON. Rows that tie on the value a table
    /// is sorted by keep this order, which is what the original's stable sort does.
    pub arrival_index: usize,
    pub name: String,
    pub job: Job,
    /// The participant's own duration, e.g. "00:34".
    pub duration_text: String,
    pub duration_seconds: f64,

    pub damage: Amount,
    pub damage_per_second: Reported,
    pub hits: Amount,
    pub misses: Amount,
    pub swings: Amount,
    pub critical_hits: Amount,
    pub direct_hits: Amount,
    pub critical_direct_hits: Amount,
    pub damage_taken: Amount,

    pub heals: Amount,
    /// Everything: HP restored + overheal + shields. See `healing()`.
    pub healed: Amount,
    pub heal_per_second: Reported,
    pub cures: Amount,
    pub critical_heals: Amount,
    pub heals_taken: Amount,
    pub damage_shield: Amount,
    pub over_heal: Amount,
    pub absorb_heal: Amount,

    pub damage_per_second_last_10_seconds: Reported,
    pub damage_per_second_last_30_seconds: Reported,
    pub damage_per_second_last_60_seconds: Reported,
    pub damage_per_second_last_180_seconds: Reported,

    /// The action of the strongest hit ("Shieldsplitter"); the amount is
    /// `strongest_hit_amount`. Empty when there was none. (ACT sends "Shieldsplitter-1819".)
    pub strongest_hit_name: String,
    pub strongest_hit_amount: Amount,
    /// The action of the strongest heal ("Divine Veil (*)"); the amount is
    /// `strongest_heal_amount`. Empty when there was none.
    pub strongest_heal_name: String,
    pub strongest_heal_amount: Amount,


    pub avoided_hits: Amount,
    pub mana_restored: Amount,
    pub parry_percent: Reported,
    pub block_percent: Reported,
    pub deaths: Amount,

    /// After pets are merged: the part of `damage` that came from pets. 0 otherwise.
    /// For drawing the pet's segment of the DPS bar.
    pub pet_damage: Amount,
    /// After pets are merged: the part of the effective healing that came from pets.
    /// For drawing the pet's segment of the HPS bar.
    pub pet_effective_healed: Amount,
}

impl Stats {
    /// Damage per second over the participant's *own* active time (ACT's `dps`, the overlay's "P.DPS"). A time of 0 counts as
    /// 1 second, as in the original.
    pub fn personal_damage_per_second(&self) -> Rate {
        let seconds = if self.duration_seconds == 0.0 { 1.0 } else { self.duration_seconds };
        Rate::per_second(self.damage, seconds)
    }

    /// Healing split into what was restored, overhealed and shielded.
    pub fn healing(&self) -> Healing {
        Healing { healed: self.healed, over_heal: self.over_heal, damage_shield: self.damage_shield }
    }

    /// ACT's `critheal%`, rounded like the original; 0 when there were no heals.
    pub fn critical_heal_share(&self) -> Percent {
        Percent::share(self.critical_heals, self.heals)
    }

    pub fn absorb_pet(&mut self, pet: &Stats) {
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
#[path = "../../../tests/unit/domain/combat/stats.rs"]
mod tests;
