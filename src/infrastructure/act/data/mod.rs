//! Everything that turns bytes from ACT into typed Rust values, using serde (infrastructure).
//!
//! ACT is inconsistent about JSON types (numbers arrive as text such as "6,703.94" or "12%",
//! placeholders such as "---" stand for "no value", booleans arrive as "true"). Those
//! mismatches are absorbed by the custom deserializers in `lenient_values`, so the rest of the
//! program only sees clean `f64` / `String` / `bool` fields.
//!
//! * `lenient_values`        – custom serde deserializers for ACT's loose scalar values
//! * `encounter_record`      – the `Encounter` object of a combat data message
//! * `combatant_record`      – one entry of the `Combatant` object: player, pet, chocobo or LB
//! * `combat_data_message`   – the whole `CombatData` payload, and folding pets into owners
//! * `combat_data_peek`      – is it active / empty? read from the raw text, without parsing
//! * `overlay_message`       – `OverlayMessage`: every OverlayPlugin message, one enum

mod combat_data_message;
mod combat_data_peek;
mod combatant_record;
mod encounter_record;
mod lenient_values;
mod overlay_message;

pub use combat_data_message::CombatDataMessage;
pub use combat_data_peek::{peek_combat_data, CombatDataPeek, SearchableText};
pub use combatant_record::CombatantRecord;
pub use encounter_record::EncounterRecord;
pub use lenient_values::describes_a_name_not_a_number;
pub use overlay_message::{
    ChangePrimaryPlayer, MessageType, OverlayMessage, ParseOptions,
};
