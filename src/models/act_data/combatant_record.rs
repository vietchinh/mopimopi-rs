//! One entry of the `Combatant` object: a player, a pet, a limit break or an NPC.

use super::lenient_values::{lenient_number, lenient_text};
use serde::Deserialize;

/// Field names follow ACT's JSON (see the `rename` attributes); Rust names say what the value is.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct CombatantRecord {
    #[serde(default, rename = "name", deserialize_with = "lenient_text")]
    pub name: String,
    /// Job abbreviation as ACT writes it ("Sch", "Whm"); empty or "0" for pets and NPCs.
    #[serde(default, rename = "Job", deserialize_with = "lenient_text")]
    pub job_text: String,
    /// The combatant's own duration, e.g. "00:34".
    #[serde(default, rename = "duration", deserialize_with = "lenient_text")]
    pub duration_text: String,
    #[serde(default, rename = "DURATION", deserialize_with = "lenient_number")]
    pub duration_seconds: f64,

    #[serde(default, deserialize_with = "lenient_number")]
    pub damage: f64,
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
    #[serde(default, deserialize_with = "lenient_number")]
    pub healed: f64,
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

    /// Strongest hit as "Action-1,234" (or "Player-Action-1,234"); empty when there was none.
    #[serde(default, rename = "maxhit", deserialize_with = "lenient_text")]
    pub strongest_hit_text: String,
    #[serde(default, rename = "MAXHIT", deserialize_with = "lenient_number")]
    pub strongest_hit_amount: f64,
    #[serde(default, rename = "maxheal", deserialize_with = "lenient_text")]
    pub strongest_heal_text: String,
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
}
