//! Which member a pet belongs to.
//!
//! ACT always lists the local player as "YOU", but names their pets after the real
//! character: "Eos (Future Fade)" belongs to "YOU" when you are Future Fade.

use super::MemberKind;
#[cfg(test)]
use super::MemberIdentity;
use std::collections::HashSet;

/// The name ACT gives the local player's own row.
pub const LOCAL_PLAYER: &str = "YOU";

/// A member's name together with what it is. Built once per update by `plan_pet_merges`.
pub(super) struct ClassifiedMember<'a> {
    pub name: &'a str,
    pub kind: MemberKind<'a>,
}

impl<'a> ClassifiedMember<'a> {
    pub(super) fn of(name: &'a str, kind: MemberKind<'a>) -> Self {
        Self { name, kind }
    }

    #[cfg(test)]
    pub(super) fn new(member: MemberIdentity<'a>) -> Self {
        Self { name: member.name, kind: MemberKind::classify(member) }
    }
}

/// Finds owners for one update's pets. Used by `plan_pet_merges`.
pub(super) struct PetOwnership<'a> {
    member_names: HashSet<&'a str>,
    local_player_name: Option<&'a str>,
}

impl<'a> PetOwnership<'a> {
    /// `local_player_name` is the character name from `ChangePrimaryPlayer`, if it has
    /// arrived yet. Until it has, mopimopi's guess is used: if exactly one pet owner matches
    /// no member, that owner must be the local player, who is listed as "YOU" instead.
    pub(super) fn new(members: &[ClassifiedMember<'a>], local_player_name: Option<&'a str>) -> Self {
        let member_names: HashSet<&str> = members.iter().map(|member| member.name).collect();
        let pet_owner_names = members.iter().filter_map(|member| member.kind.pet_owner_name());
        let local_player_name = local_player_name
            .or_else(|| only_unmatched_owner(&member_names, pet_owner_names));
        Self { member_names, local_player_name }
    }

    /// The member (by name) that owns a pet named after `owner_name`, if it is in this update.
    pub(super) fn owner_of(&self, owner_name: &str) -> Option<&'a str> {
        if let Some(&owner) = self.member_names.get(owner_name) {
            return Some(owner);
        }
        let is_local_player = self.local_player_name == Some(owner_name);
        if is_local_player {
            return self.member_names.get(LOCAL_PLAYER).copied();
        }
        None
    }
}

/// The one owner name that matches no member, or `None` if there are zero or several.
fn only_unmatched_owner<'a>(
    member_names: &HashSet<&str>,
    pet_owner_names: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let unmatched: HashSet<&str> = pet_owner_names
        .into_iter()
        .filter(|owner_name| !member_names.contains(owner_name))
        .collect();

    match unmatched.len() {
        1 => unmatched.into_iter().next(),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/pet_ownership.rs"]
mod tests;
