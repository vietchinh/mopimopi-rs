//! The `CombatData` payload as ACT sends it: one encounter plus its combatants, in the order ACT sent them.
//! Private to the ACT edge: `Encounter::from` turns it into the domain's own model.

use super::combatant_record::CombatantRecord;
use super::encounter_record::EncounterRecord;
use super::lenient_values::lenient_bool;
use crate::domain::combat::{Encounter, Stats};
use serde::de::{Deserializer, IgnoredAny, MapAccess, Visitor};
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub(in crate::infrastructure::act) struct CombatDataMessage {
    #[serde(rename = "Encounter")]
    pub encounter: EncounterRecord,
    /// ACT sends `Combatant` as an object keyed by name, but every record repeats its name,
    /// so the keys are skipped and the records kept as a list, in the order ACT sent them.
    /// Look one up with `combatant(name)`. `sort_by_damage` puts them in display order.
    #[serde(default, rename = "Combatant", deserialize_with = "deserialize_in_arrival_order")]
    pub combatants: Vec<CombatantRecord>,
    /// True while the fight is still running.
    #[serde(default, rename = "isActive", deserialize_with = "lenient_bool")]
    pub is_encounter_active: bool,
}

/// The translation from ACT's wire shape into the domain's Encounter (an anticorruption layer): the wire type ends here.
impl From<CombatDataMessage> for Encounter {
    fn from(message: CombatDataMessage) -> Encounter {
        let stats = message.combatants.into_iter().map(Stats::from).collect();
        Encounter::from_stats(message.encounter.into(), stats, message.is_encounter_active)
    }
}

/// Parses the text of a `CombatData` message into an Encounter (the bundled sample fight uses this).
pub fn encounter_from_json(text: &str) -> Result<Encounter, serde_json::Error> {
    serde_json::from_str::<CombatDataMessage>(text).map(Encounter::from)
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
            let mut combatants = Vec::with_capacity(entries.size_hint().unwrap_or(0));
            // the key repeats the combatant's name, so it is skipped without allocating
            while entries.next_key::<IgnoredAny>()?.is_some() {
                let mut record = entries.next_value::<CombatantRecord>()?;
                record.arrival_index = combatants.len();
                combatants.push(record);
            }
            Ok(combatants)
        }
    }

    deserializer.deserialize_map(CombatantsVisitor)
}

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/combat_data_message.rs"]
mod tests;
