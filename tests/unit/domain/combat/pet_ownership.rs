use super::*;

fn classified(combatants: &[(&'static str, &'static str)]) -> Vec<ClassifiedCombatant<'static>> {
    combatants.iter().map(|&(name, job)| ClassifiedCombatant::new(CombatantIdentity { name, job })).collect()
}

#[test]
fn owner_by_name() {
    let combatants = classified(&[("YOU", "Sch"), ("Kay Lionheart", "Smn"), ("Demi-Bahamut (Kay Lionheart)", "")]);
    let ownership = PetOwnership::new(&combatants, None);
    assert_eq!(ownership.owner_of("Kay Lionheart"), Some("Kay Lionheart"));
}

#[test]
fn own_pets_go_to_you_when_the_name_is_known() {
    let combatants = classified(&[("YOU", "Sch"), ("Kay Lionheart", "Mnk"), ("Eos (Future Fade)", "")]);
    let ownership = PetOwnership::new(&combatants, Some("Future Fade"));
    assert_eq!(ownership.owner_of("Future Fade"), Some(LOCAL_PLAYER));
}

#[test]
fn guess_you_when_exactly_one_owner_is_missing() {
    let combatants = classified(&[
        ("YOU", "Sch"),
        ("Kay Lionheart", "Smn"),
        ("Eos (Future Fade)", ""),
        ("Demi-Bahamut (Kay Lionheart)", ""),
    ]);
    let ownership = PetOwnership::new(&combatants, None);
    assert_eq!(ownership.owner_of("Future Fade"), Some(LOCAL_PLAYER));
}

#[test]
fn no_guess_when_two_owners_are_missing() {
    let combatants = classified(&[("YOU", "Sch"), ("Eos (Future Fade)", ""), ("Selene (Someone Who Left)", "")]);
    let ownership = PetOwnership::new(&combatants, None);
    assert_eq!(ownership.owner_of("Future Fade"), None);
    assert_eq!(ownership.owner_of("Someone Who Left"), None);
}

#[test]
fn chocobo_owners_do_not_count_for_the_guess() {
    // The chocobo's owner is missing too, but only pets are considered.
    let combatants = classified(&[("YOU", "Sch"), ("Eos (Future Fade)", ""), ("Boco (Someone Else)", "0")]);
    let ownership = PetOwnership::new(&combatants, None);
    assert_eq!(ownership.owner_of("Future Fade"), Some(LOCAL_PLAYER));
}
