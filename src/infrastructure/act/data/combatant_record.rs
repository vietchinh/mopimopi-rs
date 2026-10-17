//! One entry of the `Combatant` object: a player, a pet, a chocobo or the Limit Break.

use super::lenient_values::{lenient_number, lenient_rate, lenient_text, lenient_action_name};
use crate::domain::combat::Figures;
use serde::Deserialize;

/// Field names follow ACT's JSON (see the `rename` attributes); Rust names say what the value is.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub(in crate::infrastructure::act) struct CombatantRecord {
    /// Position in ACT's message (0 = first). Not part of the JSON. Rows that tie on the value a table
    /// is sorted by keep this order, which is what the original's stable sort does.
    #[serde(skip)]
    pub arrival_index: usize,
    #[serde(default, rename = "name", deserialize_with = "lenient_text")]
    pub name: String,
    #[serde(default, rename = "Job", deserialize_with = "lenient_text")]
    pub job_text: String,
    /// The combatant's own duration, e.g. "00:34".
    #[serde(default, rename = "duration", deserialize_with = "lenient_text")]
    pub duration_text: String,
    #[serde(default, rename = "DURATION", deserialize_with = "lenient_number")]
    pub duration_seconds: f64,

    #[serde(default, deserialize_with = "lenient_number")]
    pub damage: f64,
    #[serde(default, rename = "encdps", deserialize_with = "lenient_rate")]
    pub damage_per_second: f64,
    #[serde(default, deserialize_with = "lenient_number")]
    pub hits: f64,
    #[serde(default, deserialize_with = "lenient_number")]
    pub misses: f64,
    #[serde(default, deserialize_with = "lenient_number")]
    pub swings: f64,
    #[serde(default, rename = "crithits", deserialize_with = "lenient_number")]
    pub critical_hits: f64,
    #[serde(default, rename = "DirectHitCount", deserialize_with = "lenient_number")]
    pub direct_hits: f64,
    #[serde(default, rename = "CritDirectHitCount", deserialize_with = "lenient_number")]
    pub critical_direct_hits: f64,
    #[serde(default, rename = "damagetaken", deserialize_with = "lenient_number")]
    pub damage_taken: f64,

    #[serde(default, deserialize_with = "lenient_number")]
    pub heals: f64,
    /// Everything: HP restored + overheal + shields. See `healing()`.
    #[serde(default, deserialize_with = "lenient_number")]
    pub healed: f64,
    #[serde(default, rename = "enchps", deserialize_with = "lenient_rate")]
    pub heal_per_second: f64,
    #[serde(default, deserialize_with = "lenient_number")]
    pub cures: f64,
    #[serde(default, rename = "critheals", deserialize_with = "lenient_number")]
    pub critical_heals: f64,
    #[serde(default, rename = "healstaken", deserialize_with = "lenient_number")]
    pub heals_taken: f64,
    #[serde(default, rename = "damageShield", deserialize_with = "lenient_number")]
    pub damage_shield: f64,
    #[serde(default, rename = "overHeal", deserialize_with = "lenient_number")]
    pub over_heal: f64,
    #[serde(default, rename = "absorbHeal", deserialize_with = "lenient_number")]
    pub absorb_heal: f64,

    #[serde(default, rename = "Last10DPS", deserialize_with = "lenient_number")]
    pub damage_per_second_last_10_seconds: f64,
    #[serde(default, rename = "Last30DPS", deserialize_with = "lenient_number")]
    pub damage_per_second_last_30_seconds: f64,
    #[serde(default, rename = "Last60DPS", deserialize_with = "lenient_number")]
    pub damage_per_second_last_60_seconds: f64,
    #[serde(default, rename = "Last180DPS", deserialize_with = "lenient_number")]
    pub damage_per_second_last_180_seconds: f64,

    /// The action of the strongest hit ("Shieldsplitter"); the amount is
    /// `strongest_hit_amount`. Empty when there was none. (ACT sends "Shieldsplitter-1819".)
    #[serde(default, rename = "maxhit", deserialize_with = "lenient_action_name")]
    pub strongest_hit_name: String,
    #[serde(default, rename = "MAXHIT", deserialize_with = "lenient_number")]
    pub strongest_hit_amount: f64,
    /// The action of the strongest heal ("Divine Veil (*)"); the amount is
    /// `strongest_heal_amount`. Empty when there was none.
    #[serde(default, rename = "maxheal", deserialize_with = "lenient_action_name")]
    pub strongest_heal_name: String,
    #[serde(default, rename = "MAXHEAL", deserialize_with = "lenient_number")]
    pub strongest_heal_amount: f64,


    #[serde(default, rename = "hitfailed", deserialize_with = "lenient_number")]
    pub avoided_hits: f64,
    #[serde(default, rename = "powerheal", deserialize_with = "lenient_number")]
    pub mana_restored: f64,
    #[serde(default, rename = "ParryPct", deserialize_with = "lenient_number")]
    pub parry_percent: f64,
    #[serde(default, rename = "BlockPct", deserialize_with = "lenient_number")]
    pub block_percent: f64,
    #[serde(default, deserialize_with = "lenient_number")]
    pub deaths: f64,

    /// After pets are merged: the part of `damage` that came from pets. 0 otherwise.
    /// For drawing the pet's segment of the DPS bar.
    #[serde(skip)]
    pub pet_damage: f64,
    /// After pets are merged: the part of the effective healing that came from pets.
    /// For drawing the pet's segment of the HPS bar.
    #[serde(skip)]
    pub pet_effective_healed: f64,
}

impl From<CombatantRecord> for Figures {
    fn from(record: CombatantRecord) -> Figures {
        Figures {
            arrival_index: record.arrival_index,
            name: record.name,
            job: record.job_text.into(),
            duration_text: record.duration_text,
            duration_seconds: record.duration_seconds,
            damage: record.damage,
            damage_per_second: record.damage_per_second,
            hits: record.hits,
            misses: record.misses,
            swings: record.swings,
            critical_hits: record.critical_hits,
            direct_hits: record.direct_hits,
            critical_direct_hits: record.critical_direct_hits,
            damage_taken: record.damage_taken,
            heals: record.heals,
            healed: record.healed,
            heal_per_second: record.heal_per_second,
            cures: record.cures,
            critical_heals: record.critical_heals,
            heals_taken: record.heals_taken,
            damage_shield: record.damage_shield,
            over_heal: record.over_heal,
            absorb_heal: record.absorb_heal,
            damage_per_second_last_10_seconds: record.damage_per_second_last_10_seconds,
            damage_per_second_last_30_seconds: record.damage_per_second_last_30_seconds,
            damage_per_second_last_60_seconds: record.damage_per_second_last_60_seconds,
            damage_per_second_last_180_seconds: record.damage_per_second_last_180_seconds,
            strongest_hit_name: record.strongest_hit_name,
            strongest_hit_amount: record.strongest_hit_amount,
            strongest_heal_name: record.strongest_heal_name,
            strongest_heal_amount: record.strongest_heal_amount,
            avoided_hits: record.avoided_hits,
            mana_restored: record.mana_restored,
            parry_percent: record.parry_percent,
            block_percent: record.block_percent,
            deaths: record.deaths,
            pet_damage: record.pet_damage,
            pet_effective_healed: record.pet_effective_healed,
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/combatant_record.rs"]
mod tests;
