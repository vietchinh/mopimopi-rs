//! The players of one encounter, sorted by damage or by healing.

use super::job_classification::PET_JOB_CODE;
use super::player::Player;
use super::LOCAL_PLAYER_ROW_NAME;
use crate::models::act_data::EncounterRecord;
use crate::common::javascript_compat::number_to_javascript_string;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RankingMetric {
    Damage,
    Healing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EncounterRanking {
    pub encounter: EncounterRecord,
    /// Players sorted best first. Pets that were merged stay in the list with `is_visible == false`.
    pub players: Vec<Player>,
    pub metric: RankingMetric,
    /// Whether pets are folded into their owners.
    pub merges_pets: bool,
    /// The best player's value; bar widths are relative to it.
    pub top_value: f64,
    /// Number of visible players.
    pub party_size: usize,
    pub is_encounter_active: bool,
    /// Identifies the encounter (title + totals) to detect duplicates in the history.
    pub encounter_key: String,
    /// Owner name shared by pets when no real player has that name (the local player, whose
    /// row is called "YOU" while pets say "(Character Name)").
    pub(super) detected_local_owner_name: String,
}

impl EncounterRanking {
    /// Ranks `players` by `metric`. With `merges_pets`, pets are folded into their owners.
    /// (`local_player_name` is the character name ACT reported; pets of that character belong to `YOU`.)
    pub fn new(
        encounter: EncounterRecord,
        players: Vec<Player>,
        is_encounter_active: bool,
        metric: RankingMetric,
        merges_pets: bool,
        local_player_name: &str,
    ) -> EncounterRanking {
        let encounter_key = format!(
            "{}{}{}",
            encounter.title,
            number_to_javascript_string(encounter.total_damage),
            number_to_javascript_string(encounter.total_healed)
        );
        let mut ranking = EncounterRanking {
            encounter,
            players,
            metric,
            merges_pets,
            top_value: 0.0,
            party_size: 0,
            is_encounter_active,
            encounter_key,
            detected_local_owner_name: String::new(),
        };
        ranking.sort_and_rank();
        if merges_pets {
            ranking.fold_strongest_actions_of_pets(local_player_name);
        }
        ranking
    }

    pub fn player_named(&self, name: &str) -> Option<&Player> {
        self.players.iter().find(|player| player.name == name)
    }

    pub fn local_player(&self) -> Option<&Player> {
        self.player_named(LOCAL_PLAYER_ROW_NAME)
    }

    pub(super) fn index_of_player(&self, name: &str) -> Option<usize> {
        self.players.iter().position(|player| player.name == name)
    }

    fn ranking_value(&self, player: &Player) -> f64 {
        let stats = if self.merges_pets { &player.merged_stats } else { &player.own_stats };
        match self.metric {
            RankingMetric::Damage => stats.damage,
            RankingMetric::Healing => stats.healed,
        }
    }

    /// Merges pets (if enabled), sorts best first, and recomputes top value, ranks and party size.
    pub(super) fn sort_and_rank(&mut self) {
        self.detected_local_owner_name = self.find_local_owner_name();
        self.update_pet_visibility_and_merges();
        self.sort_players_best_first();
        self.compute_top_value();
        self.assign_ranks();
    }

    /// The owner name that no real player carries: that is the local player's character name.
    fn find_local_owner_name(&self) -> String {
        let player_names: Vec<&str> =
            self.players.iter().filter(|player| player.pet_owner_name.is_empty()).map(|player| player.name.as_str()).collect();
        self.players
            .iter()
            .filter(|player| !player.pet_owner_name.is_empty())
            .map(|player| player.pet_owner_name.as_str())
            .filter(|owner| !player_names.contains(owner))
            .last()
            .unwrap_or("")
            .to_string()
    }

    fn update_pet_visibility_and_merges(&mut self) {
        for pet_index in 0..self.players.len() {
            let is_merged_pet = self.players[pet_index].is_pet && self.merges_pets;
            if is_merged_pet {
                self.merge_pet_into_its_owners(pet_index);
            }
            self.players[pet_index].is_visible = !is_merged_pet;
        }
    }

    fn sort_players_best_first(&mut self) {
        let mut keyed: Vec<(f64, Player)> =
            std::mem::take(&mut self.players).into_iter().map(|player| (self.ranking_value(&player), player)).collect();
        // stable sort: equal values keep ACT's arrival order
        keyed.sort_by(|left, right| right.0.partial_cmp(&left.0).unwrap_or(std::cmp::Ordering::Equal));
        self.players = keyed.into_iter().map(|(_, player)| player).collect();
    }

    /// Best value among players that own a bar (merged pets have no bar of their own).
    fn compute_top_value(&mut self) {
        let top_value = self
            .players
            .iter()
            .filter(|player| !(self.merges_pets && player.job_code == PET_JOB_CODE))
            .map(|player| self.ranking_value(player))
            .fold(0.0, f64::max);
        self.top_value = top_value;
    }

    fn assign_ranks(&mut self) {
        let mut next_rank = 0;
        for player in self.players.iter_mut().filter(|player| player.is_visible) {
            player.rank = next_rank;
            next_rank += 1;
        }
        self.party_size = next_rank;
    }
}
