//! Additive per-player counters. Pets' stats are added to their owner's in `merged_stats`.

use crate::models::act_data::CombatantRecord;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerStats {
    pub damage: f64,
    pub hits: f64,
    pub misses: f64,
    pub swings: f64,
    pub critical_hits: f64,
    pub direct_hits: f64,
    pub critical_direct_hits: f64,
    pub damage_taken: f64,
    pub heals: f64,
    pub healed: f64,
    /// Healing that was neither overheal nor a shield.
    pub effective_healed: f64,
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
}

impl PlayerStats {
    pub fn from_record(record: &CombatantRecord) -> PlayerStats {
        PlayerStats {
            damage: record.damage,
            hits: record.hits,
            misses: record.misses,
            swings: record.swings,
            critical_hits: record.critical_hits,
            direct_hits: record.direct_hits,
            critical_direct_hits: record.critical_direct_hits,
            damage_taken: record.damage_taken,
            heals: record.heals,
            healed: record.healed,
            effective_healed: record.healed - record.over_heal - record.damage_shield,
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
        }
    }

    /// Adds every counter of `other` (used to fold a pet into its owner).
    pub fn add(&mut self, other: &PlayerStats) {
        self.damage += other.damage;
        self.hits += other.hits;
        self.misses += other.misses;
        self.swings += other.swings;
        self.critical_hits += other.critical_hits;
        self.direct_hits += other.direct_hits;
        self.critical_direct_hits += other.critical_direct_hits;
        self.damage_taken += other.damage_taken;
        self.heals += other.heals;
        self.healed += other.healed;
        self.effective_healed += other.effective_healed;
        self.cures += other.cures;
        self.critical_heals += other.critical_heals;
        self.heals_taken += other.heals_taken;
        self.damage_shield += other.damage_shield;
        self.over_heal += other.over_heal;
        self.absorb_heal += other.absorb_heal;
        self.damage_per_second_last_10_seconds += other.damage_per_second_last_10_seconds;
        self.damage_per_second_last_30_seconds += other.damage_per_second_last_30_seconds;
        self.damage_per_second_last_60_seconds += other.damage_per_second_last_60_seconds;
        self.damage_per_second_last_180_seconds += other.damage_per_second_last_180_seconds;
    }
}
