use super::*;
use crate::domain::combat::ParticipantIdentity;

fn identities<'a>(pairs: &[(&'a str, &'a str)]) -> Vec<(&'a str, ParticipantKind<'a>)> {
    pairs.iter().map(|&(name, job)| (name, ParticipantKind::classify(ParticipantIdentity { name, job }))).collect()
}

fn merge(pet: &str, owner: &str) -> PetMerge {
    PetMerge { pet: pet.to_owned(), owner: owner.to_owned() }
}

#[test]
fn plans_pets_only() {
    let participants = [
        ("YOU", "Sch"),
        ("Eos (Future Fade)", ""),
        ("Kay Lionheart", "Smn"),
        ("Demi-Bahamut (Kay Lionheart)", ""),
        ("Boco (Kay Lionheart)", "0"),
        ("Limit Break", "Limit Break"),
        ("Mystery Pet (Kay Lionheart)", ""),
    ];
    let mut plan = plan_pet_merges(identities(&participants), Some("Future Fade"));
    plan.sort_by(|a, b| a.pet.cmp(&b.pet));
    assert_eq!(
        plan,
        [
            merge("Demi-Bahamut (Kay Lionheart)", "Kay Lionheart"),
            merge("Eos (Future Fade)", "YOU"),
            merge("Mystery Pet (Kay Lionheart)", "Kay Lionheart"),
        ],
        "an unrecognized owned companion defaults to a (Beastmaster) pet now, not Unknown, so it merges too"
    );
}

#[test]
fn pet_of_absent_owner_stays() {
    let plan = plan_pet_merges(
        identities(&[("YOU", "Sch"), ("Kay Lionheart", "Mnk"), ("Eos (Gone)", ""), ("Selene (Also Gone)", "")]),
        None,
    );
    assert!(plan.is_empty());
}

#[test]
fn several_pets_of_one_owner() {
    let participants = [("Kay Lionheart", "Smn"), ("Demi-Bahamut (Kay Lionheart)", ""), ("Carbuncle (Kay Lionheart)", "")];
    let mut plan = plan_pet_merges(identities(&participants), None);
    plan.sort_by(|a, b| a.pet.cmp(&b.pet));
    assert_eq!(
        plan,
        [merge("Carbuncle (Kay Lionheart)", "Kay Lionheart"), merge("Demi-Bahamut (Kay Lionheart)", "Kay Lionheart")]
    );
}
