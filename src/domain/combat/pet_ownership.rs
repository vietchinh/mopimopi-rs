//! Which participant a pet belongs to.
//!
//! ACT always lists the local player as "YOU", but names their pets after the real
//! character: "Eos (Future Fade)" belongs to "YOU" when you are Future Fade.

use std::collections::HashSet;

/// The name ACT gives the local player's own row, which is also how the original labels it.
pub const LOCAL_PLAYER: &str = "YOU";

/// One participant as the pet plan sees it: its name, and the owner it names if it is a pet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetCandidate<'a> {
    pub name: &'a str,
    pub pet_owner: Option<&'a str>,
    pub is_you: bool,
}

/// Finds owners for one update's pets. Used by `plan_pet_merges`.
pub(super) struct PetOwnership<'a> {
    participant_names: HashSet<&'a str>,
    local_player_name: Option<&'a str>,
    you_name: Option<&'a str>,
}

impl<'a> PetOwnership<'a> {
    /// `local_player_name` is the character name from `ChangePrimaryPlayer`, if it has
    /// arrived yet. Until it has, mopimopi's guess is used: if exactly one pet owner matches
    /// no participant, that owner must be the local player, who is listed as "YOU" instead.
    pub(super) fn new(participants: &[PetCandidate<'a>], local_player_name: Option<&'a str>) -> Self {
        let participant_names: HashSet<&str> = participants.iter().map(|participant| participant.name).collect();
        let pet_owner_names = participants.iter().filter_map(|participant| participant.pet_owner);
        let local_player_name = local_player_name
            .or_else(|| only_unmatched_owner(&participant_names, pet_owner_names));
        let you_name = participants.iter().find(|participant| participant.is_you).map(|participant| participant.name);
        Self { participant_names, local_player_name, you_name }
    }

    /// The participant (by name) that owns a pet named after `owner_name`, if it is in this update.
    pub(super) fn owner_of(&self, owner_name: &str) -> Option<&'a str> {
        if let Some(&owner) = self.participant_names.get(owner_name) {
            return Some(owner);
        }
        if self.local_player_name == Some(owner_name) {
            return self.you_name;
        }
        None
    }
}

/// The one owner name that matches no participant, or `None` if there are zero or several.
fn only_unmatched_owner<'a>(
    participant_names: &HashSet<&str>,
    pet_owner_names: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let unmatched: HashSet<&str> = pet_owner_names
        .into_iter()
        .filter(|owner_name| !participant_names.contains(owner_name))
        .collect();

    match unmatched.len() {
        1 => unmatched.into_iter().next(),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/pet_ownership.rs"]
mod tests;
