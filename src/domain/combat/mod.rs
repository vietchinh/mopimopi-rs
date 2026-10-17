//! Who is who in a fight, and which pets belong to which player.
//!
//! * `combatant_kind` – player, pet, chocobo, Limit Break, from a combatant's name and job
//! * `encounter`     – the Encounter aggregate, its members, value objects and events (not wired in yet)
//! * `encounter_key` – identifies one finished encounter (title + totals)
//! * `history`       – the finished encounters and the rules that span fights
//! * `healing`        – effective healing, overheal and shield shares
//! * `known_pets`     – every pet name the game uses, per job and client language
//! * `pet_ownership`  – which combatant a pet belongs to, including the local player ("YOU")
//! * `pet_merge_plan` – the list of "fold this pet into that owner" steps for one update
//! * `percent`        – percentages without ACT's rounding


mod combatant_kind;
// Not used by the app until the ACT translator builds it (phase 5); remove the allow then.
#[allow(dead_code, unused_imports)]
pub mod encounter;
mod member;
mod encounter_key;
mod history;
mod healing;
mod known_pets;
mod percent;
mod pet_merge_plan;
mod pet_ownership;

pub use combatant_kind::{CombatantIdentity, CombatantKind};
pub use encounter::{Encounter, EncounterEnded, EncounterFacts, PetView, YouChanged};
pub use encounter_key::EncounterKey;
pub use member::Member;
pub use healing::Healing;
pub use history::{History, HistoryEntry};
pub use known_pets::PetJob;
pub use percent::percent_of;
pub use pet_merge_plan::plan_pet_merges;
pub use pet_ownership::LOCAL_PLAYER;
