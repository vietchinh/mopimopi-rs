//! The `CombatData` payload: one encounter plus its combatants, in the order ACT sent them.

use super::combatant_record::CombatantRecord;
use super::encounter_record::EncounterRecord;
use super::lenient_values::lenient_bool;
use serde::de::{Deserializer, IgnoredAny, MapAccess, Visitor};
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct CombatDataMessage {
    #[serde(rename = "Encounter")]
    pub encounter: EncounterRecord,
    /// `Combatant` is a JSON object keyed by name. The order is kept because ties in the
    /// ranking are resolved by arrival order.
    #[serde(default, rename = "Combatant", deserialize_with = "deserialize_in_arrival_order")]
    pub combatants: Vec<CombatantRecord>,
    /// True while the fight is still running.
    #[serde(default, rename = "isActive", deserialize_with = "lenient_bool")]
    pub is_encounter_active: bool,
}

/// Reads a JSON object as a list of its values, ignoring the keys.
fn deserialize_in_arrival_order<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<CombatantRecord>, D::Error> {
    struct CombatantsVisitor;

    impl<'de> Visitor<'de> for CombatantsVisitor {
        type Value = Vec<CombatantRecord>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an object of combatants keyed by name")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
            let mut combatants = Vec::new();
            // the key repeats the combatant's name, so it is skipped without allocating
            while entries.next_key::<IgnoredAny>()?.is_some() {
                combatants.push(entries.next_value::<CombatantRecord>()?);
            }
            Ok(combatants)
        }
    }

    deserializer.deserialize_map(CombatantsVisitor)
}

#[cfg(test)]
#[path = "../../../tests/unit/models/act_data/combat_data_message.rs"]
mod tests;
