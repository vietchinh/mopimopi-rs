//! The `Encounter` object of a combat data message.

use super::lenient_values::{lenient_number, lenient_rate, lenient_text};
use serde::Deserialize;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct EncounterRecord {
    #[serde(default, deserialize_with = "lenient_text")]
    pub title: String,
    #[serde(default, rename = "duration", deserialize_with = "lenient_text")]
    pub duration_text: String,
    #[serde(default, rename = "DURATION", deserialize_with = "lenient_number")]
    pub duration_seconds: f64,
    #[serde(default, rename = "damage", deserialize_with = "lenient_number")]
    pub total_damage: f64,
    #[serde(default, rename = "healed", deserialize_with = "lenient_number")]
    pub total_healed: f64,
    /// ACT's `encdps` ("6106.61"), not the rounded `ENCDPS` ("6107").
    #[serde(default, rename = "encdps", deserialize_with = "lenient_rate")]
    pub damage_per_second: f64,
    /// ACT's `enchps` ("2075.26"), not the rounded `ENCHPS`.
    #[serde(default, rename = "enchps", deserialize_with = "lenient_rate")]
    pub heal_per_second: f64,
    /// ACT's whole-number `ENCDPS` ("6107"), which the top bar's "Total DPS" prints as it is.
    #[serde(default, rename = "ENCDPS", deserialize_with = "lenient_number")]
    pub damage_per_second_whole: f64,
    /// ACT's whole-number `ENCHPS`, for "Total HPS".
    #[serde(default, rename = "ENCHPS", deserialize_with = "lenient_number")]
    pub heal_per_second_whole: f64,
    #[serde(default, rename = "CurrentZoneName", deserialize_with = "lenient_text")]
    pub zone_name: String,
}
