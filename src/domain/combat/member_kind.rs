//! What a member is, decided from its name and job the way mopimopi does it.

use super::known_pets::{job_of_pet, PetJob};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemberIdentity<'a> {
    pub name: &'a str,
    pub job: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberKind<'a> {
    Player,
    Pet { owner_name: &'a str, job: PetJob },
    Chocobo { owner_name: &'a str },
    LimitBreak,
    /// Anything else, e.g. a pet missing from `known_pets`. Shown as its own row.
    Unknown,
}

impl<'a> MemberKind<'a> {
    pub fn classify(member: MemberIdentity<'a>) -> Self {
        let (base_name, owner_name) = split_owner(member.name);

        match member.job {
            "Limit Break" => Self::LimitBreak,
            "0" => match owner_name {
                Some(owner_name) => Self::Chocobo { owner_name },
                None => Self::Unknown,
            },
            // ACT sends the Limit Break gauge's job text as "Limit Break", but on some captures it
            // arrives blank instead; the name alone still identifies it.
            "" if member.name == "Limit Break" => Self::LimitBreak,
            "" => match owner_name {
                Some(owner_name) => Self::Pet { owner_name, job: job_of_pet(base_name).unwrap_or(PetJob::Beastmaster) },
                // A known pet's name with no owner in it: not something ACT sends, kept as it was.
                None if job_of_pet(base_name).is_some() => Self::Unknown,
                // The original (core.js, the end of the pet lists in the original's `Combatant` class) turns every other member with a blank
                // job and no `(owner)` in its name into Limit Break: its icon, its colour, its label when names are
                // hidden. That is how it draws trust and duty-support NPCs (which ACT sends with no job) as well as the
                // Limit Break gauge itself.
                None => Self::LimitBreak,
            },
            _ => Self::Player,
        }
    }

    pub fn pet_owner_name(self) -> Option<&'a str> {
        match self {
            Self::Pet { owner_name, .. } => Some(owner_name),
            _ => None,
        }
    }
}

/// FFXIV character names cannot contain parentheses, so the last " (" starts the owner.
fn split_owner(name: &str) -> (&str, Option<&str>) {
    match name.strip_suffix(')').and_then(|rest| rest.rsplit_once(" (")) {
        Some((base_name, owner_name)) if !owner_name.is_empty() => (base_name, Some(owner_name)),
        _ => (name, None),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/member_kind.rs"]
mod tests;
