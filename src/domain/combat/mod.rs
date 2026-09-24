//! The combat domain model: players built from ACT records, pet merging, ranking.
//!
//! * `player_stats`        – additive counters (damage, hits, heals...)
//! * `strongest_action`    – "Broil-8,765" split into action name and amount
//! * `player_role`         – tank / healer / damage / crafter / gatherer
//! * `job_classification`  – decides job, class icon, role and pet-ness of a record
//! * `pet_names`           – known pet names in every game language
//! * `derived_rates`       – DPS, HPS and percentages computed from stats
//! * `player`              – one row of the tables
//! * `encounter_ranking`   – the sorted list of players for one metric (damage or healing)
//! * `pet_merging`         – folding pets into their owners
//! * `rankings`            – builds both rankings from one ACT message

mod derived_rates;
mod encounter_ranking;
mod job_classification;
mod pet_merging;
mod pet_names;
mod player;
mod player_role;
mod player_stats;
mod rankings;
mod strongest_action;

pub use encounter_ranking::EncounterRanking;
pub use job_classification::{COMBATANT_JOB_CODE, LIMIT_BREAK_JOB_CODE, PET_JOB_CODE};
pub use player::Player;
pub use player_role::PlayerRole;
pub use rankings::{build_rankings, EncounterRankings, TableKind};
pub use strongest_action::StrongestAction;

/// Name ACT gives to the local player's row.
pub const LOCAL_PLAYER_ROW_NAME: &str = "YOU";
