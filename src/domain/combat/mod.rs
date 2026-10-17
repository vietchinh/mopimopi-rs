//! Combat: the core subdomain. Who is who in a fight, and which pets belong to which player.
//!
//! **Open host service.** Everything the rest of the program may use is re-exported from this file (the published
//! language): `Encounter` and what it answers (`rows`, `local_standing`, `ended_since`, the rates), `History`, the
//! value objects and the two events. Consumers (Formatting, the tables, the top bar, the history screen, the
//! application layer) never reach into the inner modules; `tests/unit/architecture.rs` enforces it and
//! `tests/unit/domain/combat/open_host_service.rs` has one test per consumer.
//!
//! * `participant`        – the kinds of participant an Encounter holds (Player, Npc, PlayerCompanion, LimitBreak, Unclassified)
//! * `stats`       – a participant's numbers and names
//! * `participant_kind`   – how the original tells player, pet, chocobo and Limit Break apart, from a row's name and job
//! * `encounter`     – the Encounter aggregate, its participants, value objects and events (not wired in yet)
//! * `encounter_key` – identifies one finished encounter (title + totals)
//! * `history`       – the finished encounters and the rules that span fights
//! * `healing`        – effective healing, overheal and shield shares
//! * `known_pets`     – every pet name the game uses, per job and client language
//! * `pet_ownership`  – which participant a pet belongs to, including the local player ("YOU")
//! * `pet_merge_plan` – the list of "fold this pet into that owner" steps for one update


mod participant_kind;
mod encounter;
mod stats;
mod participant;
mod encounter_key;
mod history;
mod healing;
mod known_pets;
mod pet_merge_plan;
mod pet_ownership;

pub use participant_kind::{ParticipantIdentity, ParticipantKind};
pub use encounter::{Amount, Encounter, EncounterEnded, EncounterFacts, Job, Percent, PetView, Rate, Standing, YouChanged};
pub use encounter_key::EncounterKey;
pub use stats::Stats;
pub use participant::Participant;
pub use healing::Healing;
pub use history::{History, HistoryEntry};
pub use known_pets::PetJob;
pub use pet_merge_plan::plan_pet_merges;
pub use pet_ownership::LOCAL_PLAYER;

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/open_host_service.rs"]
mod open_host_service_tests;
