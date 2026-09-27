//! The `CombatData` payload: one encounter plus its combatants, in the order ACT sent them.

use super::combatant_record::CombatantRecord;
use super::encounter_record::EncounterRecord;
use super::lenient_values::lenient_bool;
use crate::domain::combat::{plan_pet_merges, PetMerge, LOCAL_PLAYER};
use serde::de::{Deserializer, IgnoredAny, MapAccess, Visitor};
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct CombatDataMessage {
    #[serde(rename = "Encounter")]
    pub encounter: EncounterRecord,
    /// ACT sends `Combatant` as an object keyed by name, but every record repeats its name,
    /// so the keys are skipped and the records kept as a list, in the order ACT sent them.
    /// Look one up with `combatant(name)`; the GUI sorts for display.
    #[serde(default, rename = "Combatant", deserialize_with = "deserialize_in_arrival_order")]
    pub combatants: Vec<CombatantRecord>,
    /// True while the fight is still running.
    #[serde(default, rename = "isActive", deserialize_with = "lenient_bool")]
    pub is_encounter_active: bool,
}

impl CombatDataMessage {
    /// The combatant with this name ("YOU", "Kay Lionheart", "Eos (Future Fade)").
    pub fn combatant(&self, name: &str) -> Option<&CombatantRecord> {
        self.combatants.iter().find(|combatant| combatant.name == name)
    }

    /// The local player's row, if they are in this fight. ACT always lists the local
    /// player as "YOU".
    pub fn local_player(&self) -> Option<&CombatantRecord> {
        self.combatant(LOCAL_PLAYER)
    }

    /// Folds pets into their owners as planned by the domain (`plan_pet_merges`).
    /// `local_player_name` lets the player's own pets find the "YOU" row.
    pub fn merge_pets_into_owners(&mut self, local_player_name: Option<&str>) {
        let identities = self.combatants.iter().map(CombatantRecord::identity);
        let merges = plan_pet_merges(identities, local_player_name);
        if merges.is_empty() {
            return;
        }
        for merge in &merges {
            self.fold_pet_into_owner(merge);
        }
        self.combatants.sort_by(|a, b| {
            match (b.damage_per_second).partial_cmp(&(a.damage_per_second)) {
                Some(std::cmp::Ordering::Equal) | None => b.damage.partial_cmp(&a.damage).unwrap_or(std::cmp::Ordering::Equal),
                Some(order) => order,
            }
        });
    }

    /// Moves one pet's numbers into its owner and removes the pet's row. The plan only names
    /// owners that are present; should one be missing anyway, the pet's row is left as it is.
    fn fold_pet_into_owner(&mut self, merge: &PetMerge) {
        if self.combatant(&merge.owner).is_none() {
            return;
        }
        let Some(pet_index) = self.position_of(&merge.pet) else { return };
        let pet = self.combatants.remove(pet_index);
        if let Some(owner) = self.combatant_mut(&merge.owner) {
            owner.absorb_pet(&pet);
        }
    }

    fn position_of(&self, name: &str) -> Option<usize> {
        self.combatants.iter().position(|combatant| combatant.name == name)
    }

    fn combatant_mut(&mut self, name: &str) -> Option<&mut CombatantRecord> {
        self.combatants.iter_mut().find(|combatant| combatant.name == name)
    }
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
                combatants.push(entries.next_value::<CombatantRecord>()?);
            }
            Ok(combatants)
        }
    }

    deserializer.deserialize_map(CombatantsVisitor)
}

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/combat_data_message.rs"]
mod tests;
