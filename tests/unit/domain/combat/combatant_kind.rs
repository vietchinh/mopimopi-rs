use super::*;

fn classify<'a>(name: &'a str, job: &'a str) -> CombatantKind<'a> {
    CombatantKind::classify(CombatantIdentity { name, job })
}

#[test]
fn players_by_job() {
    assert_eq!(classify("Gwenn Windspeaker", "Mnk"), CombatantKind::Player);
    assert_eq!(classify("YOU", "Drk"), CombatantKind::Player);
}

#[test]
fn known_pets_in_any_language() {
    assert_eq!(
        classify("Eos (Future Fade)", ""),
        CombatantKind::Pet { owner_name: "Future Fade", job: PetJob::Scholar }
    );
    assert_eq!(
        classify("フェアリー・エオス (Future Fade)", ""),
        CombatantKind::Pet { owner_name: "Future Fade", job: PetJob::Scholar }
    );
    assert_eq!(
        classify("Cu Sith (YOU)", ""),
        CombatantKind::Pet { owner_name: "YOU", job: PetJob::Beastmaster }
    );
}

#[test]
fn chocobo_limit_break_and_unknown() {
    assert_eq!(
        classify("Boco (Future Fade)", "0"),
        CombatantKind::Chocobo { owner_name: "Future Fade" }
    );
    assert_eq!(classify("Limit Break", "Limit Break"), CombatantKind::LimitBreak);
    assert_eq!(classify("Limit Break", ""), CombatantKind::LimitBreak);
    assert_eq!(classify("Mystery Pet (Future Fade)", ""), CombatantKind::Unknown);
}

#[test]
fn owner_is_needed_for_pets_and_chocobos() {
    assert_eq!(classify("Eos", ""), CombatantKind::Unknown);
    assert_eq!(classify("Boco", "0"), CombatantKind::Unknown);
}

#[test]
fn pet_owner_name_only_for_pets() {
    assert_eq!(classify("Eos (Future Fade)", "").pet_owner_name(), Some("Future Fade"));
    assert_eq!(classify("Boco (Future Fade)", "0").pet_owner_name(), None);
    assert_eq!(classify("YOU", "Sch").pet_owner_name(), None);
}
