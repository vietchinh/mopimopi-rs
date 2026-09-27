//! Who is who in a fight, and which pets belong to which player.
//!
//! * `combatant_kind` – player, pet, chocobo, Limit Break, from a combatant's name and job
//! * `healing`        – effective healing, overheal and shield shares
//! * `known_pets`     – every pet name the game uses, per job and client language
//! * `pet_ownership`  – which combatant a pet belongs to, including the local player ("YOU")
//! * `pet_merge_plan` – the list of "fold this pet into that owner" steps for one update
//! * `percent`        – percentages without ACT's rounding

mod combatant_kind;
mod healing;
mod known_pets;
mod percent;
mod pet_merge_plan;
mod pet_ownership;
mod rankings;

pub use combatant_kind::{CombatantIdentity, CombatantKind};
pub use healing::Healing;
pub use known_pets::PetJob;
pub use percent::percent_of;
pub use pet_merge_plan::{plan_pet_merges, PetMerge};
pub use pet_ownership::LOCAL_PLAYER;

// pub use encounter_ranking::EncounterRanking;
// pub use job_classification::{COMBATANT_JOB_CODE, LIMIT_BREAK_JOB_CODE, PET_JOB_CODE};
// pub use player::Player;
// pub use player_role::PlayerRole;
pub use rankings::{TableKind};
// pub use strongest_action::StrongestAction;
