//! One row of the tables: a player, pet, limit break or owned combatant.

use super::derived_rates::DerivedRates;
use super::job_classification::classify;
use super::player_role::PlayerRole;
use super::player_stats::PlayerStats;
use super::strongest_action::StrongestAction;
use crate::models::act_data::{CombatantRecord, EncounterRecord};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Player {
    pub name: String,
    /// Job as used by the overlay: real job text, or "AVA" (pet), "LMB", "CBO", "0".
    pub job_code: String,
    /// Icon / colour key (upper case job, base classes mapped to their job).
    pub class_code: String,
    pub role: PlayerRole,
    pub is_pet: bool,
    pub pet_owner_name: String,
    /// Position in the ranking, counting only visible players (0 = first).
    pub rank: usize,
    /// False for pets that were folded into their owner.
    pub is_visible: bool,
    pub personal_duration_text: String,
    pub personal_duration_seconds: f64,
    /// Stats exactly as ACT sent them.
    pub own_stats: PlayerStats,
    /// Own stats plus those of merged pets (equals `own_stats` when pets are not merged).
    pub merged_stats: PlayerStats,
    pub(super) merged_pets: Vec<(String, PlayerStats)>,
    pub own_strongest_hit: StrongestAction,
    pub merged_strongest_hit: StrongestAction,
    pub own_strongest_heal: StrongestAction,
    pub merged_strongest_heal: StrongestAction,
    pub avoided_hits: f64,
    pub mana_restored: f64,
    pub parry_percent: f64,
    pub block_percent: f64,
    pub deaths: f64,
    pub rates: DerivedRates,
}

impl Player {
    pub fn from_record(record: &CombatantRecord, encounter: &EncounterRecord) -> Player {
        let classification = classify(&record.name, &record.job_text);
        let own_stats = PlayerStats::from_record(record);
        let strongest_hit = StrongestAction::from_act_fields(&record.strongest_hit_text, record.strongest_hit_amount);
        let strongest_heal = StrongestAction::from_act_fields(&record.strongest_heal_text, record.strongest_heal_amount);

        let mut player = Player {
            name: record.name.clone(),
            job_code: classification.job_code,
            class_code: classification.class_code,
            role: classification.role,
            is_pet: classification.is_pet,
            pet_owner_name: classification.pet_owner_name,
            rank: 0,
            is_visible: true,
            personal_duration_text: record.duration_text.clone(),
            personal_duration_seconds: record.duration_seconds,
            merged_stats: own_stats.clone(),
            own_stats,
            merged_pets: Vec::new(),
            merged_strongest_hit: strongest_hit.clone(),
            own_strongest_hit: strongest_hit,
            merged_strongest_heal: strongest_heal.clone(),
            own_strongest_heal: strongest_heal,
            avoided_hits: record.avoided_hits,
            mana_restored: record.mana_restored,
            parry_percent: record.parry_percent,
            block_percent: record.block_percent,
            deaths: record.deaths,
            rates: DerivedRates::default(),
        };
        player.recalculate_rates(encounter);
        player
    }

    pub fn recalculate_rates(&mut self, encounter: &EncounterRecord) {
        self.rates = DerivedRates::calculate(&self.merged_stats, self.personal_duration_seconds, encounter);
    }

    /// What a pet contributes to its owner: its name and own stats.
    pub(super) fn clone_name_and_own_stats(&self) -> (String, PlayerStats) {
        (self.name.clone(), self.own_stats.clone())
    }

    /// Adds (or refreshes) one pet's stats to this player's merged stats.
    pub(super) fn merge_pet_stats(&mut self, pet_name: &str, pet_stats: &PlayerStats, encounter: &EncounterRecord) {
        self.merged_stats = self.own_stats.clone();
        match self.merged_pets.iter_mut().find(|(known_name, _)| known_name == pet_name) {
            Some((_, known_stats)) => *known_stats = pet_stats.clone(),
            None => self.merged_pets.push((pet_name.to_string(), pet_stats.clone())),
        }
        for (_, stats) in &self.merged_pets {
            self.merged_stats.add(stats);
        }
        self.recalculate_rates(encounter);
    }

}
