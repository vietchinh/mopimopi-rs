//! Rates and percentages computed from a player's (possibly pet-merged) stats.

use super::player_stats::PlayerStats;
use crate::models::act_data::EncounterRecord;
use crate::common::javascript_compat::round_to_two_decimals;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DerivedRates {
    /// Damage per second of the player's own active time.
    pub damage_per_second: f64,
    /// Damage per second of the whole encounter.
    pub encounter_damage_per_second: f64,
    pub heal_per_second: f64,
    pub encounter_heal_per_second: f64,
    /// Share of the encounter's total damage, 0-100.
    pub damage_percent: f64,
    pub healed_percent: f64,
    pub over_heal_percent: f64,
    pub critical_hit_percent: f64,
    pub direct_hit_percent: f64,
    pub critical_direct_hit_percent: f64,
    pub critical_heal_percent: f64,
    /// Hits divided by swings, 0-100.
    pub accuracy_percent: f64,
}

impl DerivedRates {
    pub fn calculate(stats: &PlayerStats, personal_duration_seconds: f64, encounter: &EncounterRecord) -> DerivedRates {
        let personal_seconds = if personal_duration_seconds == 0.0 { 1.0 } else { personal_duration_seconds };
        DerivedRates {
            damage_per_second: clean(stats.damage / personal_seconds),
            encounter_damage_per_second: clean(stats.damage / encounter.duration_seconds),
            heal_per_second: clean(stats.healed / personal_seconds),
            encounter_heal_per_second: clean(stats.healed / encounter.duration_seconds),
            damage_percent: clean(stats.damage / encounter.total_damage * 100.0),
            healed_percent: clean(stats.healed / encounter.total_healed * 100.0),
            over_heal_percent: clean(stats.over_heal / stats.healed * 100.0),
            critical_hit_percent: clean(stats.critical_hits / stats.hits * 100.0),
            direct_hit_percent: clean(stats.direct_hits / stats.hits * 100.0),
            critical_direct_hit_percent: clean(stats.critical_direct_hits / stats.hits * 100.0),
            critical_heal_percent: clean(stats.critical_heals / stats.heals * 100.0),
            accuracy_percent: clean(stats.hits / stats.swings * 100.0),
        }
    }
}

/// Division by zero gives NaN (shown as 0) or infinity (shown as "∞"); values are rounded to 2 decimals.
fn clean(value: f64) -> f64 {
    if value.is_nan() { 0.0 } else { round_to_two_decimals(value) }
}
