//! Which players a table shows.

use crate::domain::combat::{EncounterRanking, Player, PlayerRole, TableKind, COMBATANT_JOB_CODE, PET_JOB_CODE};
use crate::domain::settings::Settings;

/// The job-filter options ("DPS_T" = tanks in the DPS table, "HPS_H" = healers in the HPS table ...).
fn passes_job_filter(settings: &Settings, table: TableKind, player: &Player) -> bool {
    let filter_enabled = |suffix: &str| settings.option_enabled(&format!("{}_{suffix}", table.short_label()));
    (filter_enabled("T") && player.role == PlayerRole::Tank)
        || (filter_enabled("H") && player.role == PlayerRole::Healer)
        || (filter_enabled("D") && player.role == PlayerRole::Damage)
        || (filter_enabled("C") && player.job_code == COMBATANT_JOB_CODE)
        || (filter_enabled("M") && matches!(player.role, PlayerRole::Crafter | PlayerRole::Gatherer))
}

/// Players in ranking order, without merged pets and without those the job filter hides.
pub(super) fn visible_players<'a>(settings: &Settings, ranking: &'a EncounterRanking, table: TableKind) -> Vec<&'a Player> {
    let pets_are_merged = settings.option_enabled("pets");
    ranking
        .players
        .iter()
        .filter(|player| !(pets_are_merged && player.job_code == PET_JOB_CODE))
        .filter(|player| !player.class_code.is_empty() && passes_job_filter(settings, table, player))
        .collect()
}
