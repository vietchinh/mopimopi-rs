//! Decides, for one update, which pets fold into which owners. It only names participants;
//! adding up the numbers is done by the participant record (`Participant::absorb_pet`).

use super::pet_ownership::{PetCandidate, PetOwnership};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetMerge {
    pub pet: String,
    pub owner: String,
}

/// Chocobos, unknown pets and pets whose owner is not in this update are left alone.
pub fn plan_pet_merges<'a>(participants: impl IntoIterator<Item = PetCandidate<'a>>, local_player_name: Option<&'a str>) -> Vec<PetMerge> {
    let candidates: Vec<PetCandidate> = participants.into_iter().collect();
    let ownership = PetOwnership::new(&candidates, local_player_name);

    candidates
        .iter()
        .filter_map(|participant| {
            let owner = ownership.owner_of(participant.pet_owner?)?;
            Some(PetMerge { pet: participant.name.to_owned(), owner: owner.to_owned() })
        })
        .collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/pet_merge_plan.rs"]
mod tests;
