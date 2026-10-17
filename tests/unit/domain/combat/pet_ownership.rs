use super::*;

/// (name, owner of the pet it names, is it you)
fn candidates(participants: &[(&'static str, Option<&'static str>)]) -> Vec<PetCandidate<'static>> {
    participants.iter().map(|&(name, pet_owner)| PetCandidate { name, pet_owner, is_you: name == LOCAL_PLAYER }).collect()
}

#[test]
fn owner_by_name() {
    let participants = candidates(&[("YOU", None), ("Kay Lionheart", None), ("Demi-Bahamut (Kay Lionheart)", Some("Kay Lionheart"))]);
    let ownership = PetOwnership::new(&participants, None);
    assert_eq!(ownership.owner_of("Kay Lionheart"), Some("Kay Lionheart"));
}

#[test]
fn own_pets_go_to_you_when_the_name_is_known() {
    let participants = candidates(&[("YOU", None), ("Kay Lionheart", None), ("Eos (Future Fade)", Some("Future Fade"))]);
    let ownership = PetOwnership::new(&participants, Some("Future Fade"));
    assert_eq!(ownership.owner_of("Future Fade"), Some(LOCAL_PLAYER));
}

#[test]
fn guess_you_when_exactly_one_owner_is_missing() {
    let participants = candidates(&[
        ("YOU", None),
        ("Kay Lionheart", None),
        ("Eos (Future Fade)", Some("Future Fade")),
        ("Demi-Bahamut (Kay Lionheart)", Some("Kay Lionheart")),
    ]);
    let ownership = PetOwnership::new(&participants, None);
    assert_eq!(ownership.owner_of("Future Fade"), Some(LOCAL_PLAYER));
}

#[test]
fn no_guess_when_two_owners_are_missing() {
    let participants = candidates(&[("YOU", None), ("Eos (Future Fade)", Some("Future Fade")), ("Selene (Someone Who Left)", Some("Someone Who Left"))]);
    let ownership = PetOwnership::new(&participants, None);
    assert_eq!(ownership.owner_of("Future Fade"), None);
    assert_eq!(ownership.owner_of("Someone Who Left"), None);
}

#[test]
fn chocobo_owners_do_not_count_for_the_guess() {
    // The chocobo's owner is missing too, but only pets name an owner here: a chocobo has none to offer.
    let participants = candidates(&[("YOU", None), ("Eos (Future Fade)", Some("Future Fade")), ("Boco (Someone Else)", None)]);
    let ownership = PetOwnership::new(&participants, None);
    assert_eq!(ownership.owner_of("Future Fade"), Some(LOCAL_PLAYER));
}
