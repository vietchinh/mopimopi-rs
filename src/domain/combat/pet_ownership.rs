//! Which combatant a pet belongs to.
//!
//! ACT always lists the local player as "YOU", but names their pets after the real
//! character: "Eos (Future Fade)" belongs to "YOU" when you are Future Fade.

use super::{CombatantIdentity, CombatantKind};
use std::collections::HashSet;

/// The name ACT gives the local player's own row.
pub const LOCAL_PLAYER: &str = "YOU";

/// A combatant's name together with what it is. Built once per update by `plan_pet_merges`.
pub(super) struct ClassifiedCombatant<'a> {
    pub name: &'a str,
    pub kind: CombatantKind<'a>,
}

impl<'a> ClassifiedCombatant<'a> {
    pub(super) fn new(combatant: CombatantIdentity<'a>) -> Self {
        Self { name: combatant.name, kind: CombatantKind::classify(combatant) }
    }
}

/// Finds owners for one update's pets. Used by `plan_pet_merges`.
pub(super) struct PetOwnership<'a> {
    combatant_names: HashSet<&'a str>,
    local_player_name: Option<&'a str>,
}

impl<'a> PetOwnership<'a> {
    /// `local_player_name` is the character name from `ChangePrimaryPlayer`, if it has
    /// arrived yet. Until it has, mopimopi's guess is used: if exactly one pet owner matches
    /// no combatant, that owner must be the local player, who is listed as "YOU" instead.
    pub(super) fn new(combatants: &[ClassifiedCombatant<'a>], local_player_name: Option<&'a str>) -> Self {
        let combatant_names: HashSet<&str> = combatants.iter().map(|combatant| combatant.name).collect();
        let pet_owner_names = combatants.iter().filter_map(|combatant| combatant.kind.pet_owner_name());
        let local_player_name = local_player_name
            .or_else(|| only_unmatched_owner(&combatant_names, pet_owner_names));
        Self { combatant_names, local_player_name }
    }

    /// The combatant (by name) that owns a pet named after `owner_name`, if it is in this update.
    pub(super) fn owner_of(&self, owner_name: &str) -> Option<&'a str> {
        if let Some(&owner) = self.combatant_names.get(owner_name) {
            return Some(owner);
        }
        let is_local_player = self.local_player_name == Some(owner_name);
        if is_local_player {
            return self.combatant_names.get(LOCAL_PLAYER).copied();
        }
        None
    }
}

/// The one owner name that matches no combatant, or `None` if there are zero or several.
fn only_unmatched_owner<'a>(
    combatant_names: &HashSet<&str>,
    pet_owner_names: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let unmatched: HashSet<&str> = pet_owner_names
        .into_iter()
        .filter(|owner_name| !combatant_names.contains(owner_name))
        .collect();

    match unmatched.len() {
        1 => unmatched.into_iter().next(),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/pet_ownership.rs"]
mod tests;
