//! The `Encounter` object of a combat data message.

use super::lenient_values::{lenient_number, lenient_text};
use serde::Deserialize;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct EncounterRecord {
    /// Boss or target name ("Striking Dummy"); ACT sends "Encounter" when there is none.
    #[serde(default, deserialize_with = "lenient_text")]
    pub title: String,
    /// Display duration such as "00:35".
    #[serde(default, rename = "duration", deserialize_with = "lenient_text")]
    pub duration_text: String,
    #[serde(default, rename = "DURATION", deserialize_with = "lenient_number")]
    pub duration_seconds: f64,
    #[serde(default, rename = "damage", deserialize_with = "lenient_number")]
    pub total_damage: f64,
    #[serde(default, rename = "healed", deserialize_with = "lenient_number")]
    pub total_healed: f64,
    #[serde(default, rename = "ENCDPS", deserialize_with = "lenient_number")]
    pub damage_per_second: f64,
    #[serde(default, rename = "ENCHPS", deserialize_with = "lenient_number")]
    pub heal_per_second: f64,
    #[serde(default, rename = "CurrentZoneName", deserialize_with = "lenient_text")]
    pub zone_name: String,
}
