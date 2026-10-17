use super::*;

/// (name, owner of the pet it names): the edge has already decided which rows are pets.
fn candidates<'a>(rows: &[(&'a str, Option<&'a str>)]) -> Vec<PetCandidate<'a>> {
    rows.iter().map(|&(name, pet_owner)| PetCandidate { name, pet_owner, is_you: name == "YOU" }).collect()
}

fn merge(pet: &str, owner: &str) -> PetMerge {
    PetMerge { pet: pet.to_owned(), owner: owner.to_owned() }
}

#[test]
fn plans_pets_only() {
    let participants = [
        ("YOU", None),
        ("Eos (Future Fade)", Some("Future Fade")),
        ("Kay Lionheart", None),
        ("Demi-Bahamut (Kay Lionheart)", Some("Kay Lionheart")),
        ("Boco (Kay Lionheart)", None),
        ("Limit Break", None),
        ("Mystery Pet (Kay Lionheart)", Some("Kay Lionheart")),
    ];
    let mut plan = plan_pet_merges(candidates(&participants), Some("Future Fade"));
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
        candidates(&[("YOU", None), ("Kay Lionheart", None), ("Eos (Gone)", Some("Gone")), ("Selene (Also Gone)", Some("Also Gone"))]),
        None,
    );
    assert!(plan.is_empty());
}

#[test]
fn several_pets_of_one_owner() {
    let participants = [("Kay Lionheart", None), ("Demi-Bahamut (Kay Lionheart)", Some("Kay Lionheart")), ("Carbuncle (Kay Lionheart)", Some("Kay Lionheart"))];
    let mut plan = plan_pet_merges(candidates(&participants), None);
    plan.sort_by(|a, b| a.pet.cmp(&b.pet));
    assert_eq!(
        plan,
        [merge("Carbuncle (Kay Lionheart)", "Kay Lionheart"), merge("Demi-Bahamut (Kay Lionheart)", "Kay Lionheart")]
    );
}
