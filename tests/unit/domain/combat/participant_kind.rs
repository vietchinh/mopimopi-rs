use super::*;

fn classify<'a>(name: &'a str, job: &'a str) -> ParticipantKind<'a> {
    ParticipantKind::classify(ParticipantIdentity { name, job })
}

#[test]
fn players_by_job() {
    assert_eq!(classify("Gwenn Windspeaker", "Mnk"), ParticipantKind::Player);
    assert_eq!(classify("YOU", "Drk"), ParticipantKind::Player);
}

#[test]
fn known_pets_in_any_language() {
    assert_eq!(
        classify("Eos (Future Fade)", ""),
        ParticipantKind::Pet { owner_name: "Future Fade", job: PetJob::Scholar }
    );
    assert_eq!(
        classify("フェアリー・エオス (Future Fade)", ""),
        ParticipantKind::Pet { owner_name: "Future Fade", job: PetJob::Scholar }
    );
    assert_eq!(
        classify("Cu Sith (YOU)", ""),
        ParticipantKind::Pet { owner_name: "YOU", job: PetJob::Beastmaster }
    );
}

#[test]
fn chocobo_limit_break_and_unknown() {
    assert_eq!(
        classify("Boco (Future Fade)", "0"),
        ParticipantKind::Chocobo { owner_name: "Future Fade" }
    );
    assert_eq!(classify("Limit Break", "Limit Break"), ParticipantKind::LimitBreak);
    assert_eq!(classify("Limit Break", ""), ParticipantKind::LimitBreak);
    assert_eq!(
        classify("Mystery Pet (Future Fade)", ""),
        ParticipantKind::Pet { owner_name: "Future Fade", job: PetJob::Beastmaster },
        "an owned companion with an unrecognized name defaults to Beastmaster, not Unknown"
    );
}

#[test]
fn a_blank_job_without_an_owner_is_limit_break_like_in_the_original() {
    // trust / duty-support NPCs: ACT sends no job, and there is no "(owner)" in the name
    assert_eq!(classify("Aw'aern", ""), ParticipantKind::LimitBreak);
    assert_eq!(classify("Aw'zdei", ""), ParticipantKind::LimitBreak);
    // ... but a name with an owner is a pet, and a known pet's name on its own stays as it was
    assert!(matches!(classify("Aw'aern (Player)", ""), ParticipantKind::Pet { .. }));
    assert_eq!(classify("Eos", ""), ParticipantKind::Unknown);
}

#[test]
fn owner_is_needed_for_pets_and_chocobos() {
    assert_eq!(classify("Eos", ""), ParticipantKind::Unknown);
    assert_eq!(classify("Boco", "0"), ParticipantKind::Unknown);
}

#[test]
fn pet_owner_name_only_for_pets() {
    assert_eq!(classify("Eos (Future Fade)", "").pet_owner_name(), Some("Future Fade"));
    assert_eq!(classify("Boco (Future Fade)", "0").pet_owner_name(), None);
    assert_eq!(classify("YOU", "Sch").pet_owner_name(), None);
}
