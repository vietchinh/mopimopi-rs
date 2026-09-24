//! Builds the two rankings (by damage and by healing) the tables need from one ACT message.

use super::encounter_ranking::{EncounterRanking, RankingMetric};
use super::player::Player;
use crate::models::act_data::CombatDataMessage;

/// Both views of one encounter.
#[derive(Clone, Debug, PartialEq)]
pub struct EncounterRankings {
    pub by_damage: EncounterRanking,
    pub by_healing: EncounterRanking,
}

impl EncounterRankings {
    pub fn ranking_for_table(&self, table: TableKind) -> &EncounterRanking {
        match table {
            TableKind::Damage => &self.by_damage,
            TableKind::Healing => &self.by_healing,
        }
    }
}

/// Which of the two tables is meant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableKind {
    Damage,
    Healing,
}

impl TableKind {
    /// "DPS" or "HPS": the prefix used in element ids, column flags and setting keys.
    pub fn short_label(self) -> &'static str {
        match self {
            TableKind::Damage => "DPS",
            TableKind::Healing => "HPS",
        }
    }
}

/// Players of the message, built once and shared by both rankings. Combatants without a class
/// (unrecognised NPCs) are dropped.
fn build_players(message: &CombatDataMessage) -> Vec<Player> {
    message
        .combatants
        .iter()
        .map(|record| Player::from_record(record, &message.encounter))
        .filter(|player| !player.class_code.is_empty())
        .collect()
}

pub fn build_rankings(message: &CombatDataMessage, merge_pets: bool, local_player_name: &str) -> EncounterRankings {
    let players = build_players(message);
    let ranking = |metric, players| {
        EncounterRanking::new(
            message.encounter.clone(),
            players,
            message.is_encounter_active,
            metric,
            merge_pets,
            local_player_name,
        )
    };
    // the second ranking takes the players by move, so only one copy is made
    let by_damage = ranking(RankingMetric::Damage, players.clone());
    let by_healing = ranking(RankingMetric::Healing, players);
    EncounterRankings { by_damage, by_healing }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/rankings.rs"]
mod tests;
