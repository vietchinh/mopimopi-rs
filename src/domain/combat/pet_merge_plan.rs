//! Decides, for one update, which pets fold into which owners. It only names members;
//! adding up the numbers is done by the member record (`Member::absorb_pet`).

use super::pet_ownership::{ClassifiedMember, PetOwnership};
use super::MemberKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetMerge {
    pub pet: String,
    pub owner: String,
}

/// Chocobos, unknown pets and pets whose owner is not in this update are left alone.
pub fn plan_pet_merges<'a>(
    members: impl IntoIterator<Item = (&'a str, MemberKind<'a>)>,
    local_player_name: Option<&'a str>,
) -> Vec<PetMerge> {
    let classified: Vec<ClassifiedMember> = members.into_iter().map(|(name, kind)| ClassifiedMember::of(name, kind)).collect();
    let ownership = PetOwnership::new(&classified, local_player_name);

    classified
        .iter()
        .filter_map(|member| {
            let owner = ownership.owner_of(member.kind.pet_owner_name()?)?;
            Some(PetMerge { pet: member.name.to_owned(), owner: owner.to_owned() })
        })
        .collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/pet_merge_plan.rs"]
mod tests;
