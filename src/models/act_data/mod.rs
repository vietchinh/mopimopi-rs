//! Everything that turns bytes from ACT into typed Rust values, using serde.
//!
//! ACT is inconsistent about JSON types (numbers arrive as text such as "6,703.94" or "12%",
//! placeholders such as "---" stand for "no value", booleans arrive as "true"). Those
//! mismatches are absorbed by the custom deserializers in `lenient_values`, so the rest of the
//! program only sees clean `f64` / `String` / `bool` fields.
//!
//! * `lenient_values`        – custom serde deserializers for ACT's loose scalar values
//! * `encounter_record`      – the `Encounter` object of a combat data message
//! * `combatant_record`      – one entry of the `Combatant` object (one player, pet or NPC)
//! * `combat_data_message`   – the whole `CombatData` payload (encounter + ordered combatants)
//! * `incoming_message`      – every WebSocket message shape (OverlayPlugin and MiniParse)

mod combat_data_message;
mod combatant_record;
mod encounter_record;
mod incoming_message;
mod lenient_values;

pub use combat_data_message::CombatDataMessage;
pub use combatant_record::CombatantRecord;
pub use encounter_record::EncounterRecord;
pub use incoming_message::{parse_bare_combat_data, parse_incoming_message, ActEvent};
pub use lenient_values::describes_a_name_not_a_number;
