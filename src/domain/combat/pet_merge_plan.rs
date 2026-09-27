//! Decides, for one update, which pets fold into which owners. It only names combatants;
//! adding up the numbers is done by the combatant record (`CombatantRecord::absorb_pet`).

use super::pet_ownership::{ClassifiedCombatant, PetOwnership};
use super::CombatantIdentity;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetMerge {
    pub pet: String,
    pub owner: String,
}

/// Chocobos, unknown pets and pets whose owner is not in this update are left alone.
pub fn plan_pet_merges<'a>(
    combatants: impl IntoIterator<Item = CombatantIdentity<'a>>,
    local_player_name: Option<&'a str>,
) -> Vec<PetMerge> {
    let classified: Vec<ClassifiedCombatant> = combatants.into_iter().map(ClassifiedCombatant::new).collect();
    let ownership = PetOwnership::new(&classified, local_player_name);

    classified
        .iter()
        .filter_map(|combatant| {
            let owner = ownership.owner_of(combatant.kind.pet_owner_name()?)?;
            Some(PetMerge { pet: combatant.name.to_owned(), owner: owner.to_owned() })
        })
        .collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/pet_merge_plan.rs"]
mod tests;
