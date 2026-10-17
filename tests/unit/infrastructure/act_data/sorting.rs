use super::*;
use crate::domain::combat::CompanionKind;

fn sorted(name: &str, job: &str) -> Participant {
    sort(Stats { name: name.into(), job: job.to_string().into(), ..Stats::default() })
}

#[test]
fn players_by_job_and_you_by_name() {
    assert!(matches!(sorted("Gwenn Windspeaker", "Mnk"), Participant::Player(_)));
    assert!(matches!(sorted("YOU", "Drk"), Participant::You(_)));
    assert!(matches!(sorted("YOU", ""), Participant::You(_)), "ACT's own name for the local player decides, whatever the job text");
}

#[test]
fn known_pets_in_any_language() {
    for name in ["Eos (Future Fade)", "フェアリー・エオス (Future Fade)"] {
        assert_eq!(sorted(name, "").companion_of(), Some(("Future Fade", CompanionKind::Pet(PetJob::Scholar))), "{name}");
    }
    assert_eq!(sorted("Cu Sith (YOU)", "").companion_of(), Some(("YOU", CompanionKind::Pet(PetJob::Beastmaster))));
}

#[test]
fn chocobo_limit_break_and_unknown() {
    assert_eq!(sorted("Boco (Future Fade)", "0").companion_of(), Some(("Future Fade", CompanionKind::Chocobo)));
    assert!(matches!(sorted("Limit Break", "Limit Break"), Participant::LimitBreak(_)));
    assert!(matches!(sorted("Limit Break", ""), Participant::LimitBreak(_)));
    assert_eq!(
        sorted("Mystery Pet (Future Fade)", "").companion_of(),
        Some(("Future Fade", CompanionKind::Pet(PetJob::Beastmaster))),
        "an owned companion with an unrecognized name defaults to Beastmaster, not Unclassified"
    );
}

#[test]
fn a_blank_job_without_an_owner_is_an_npc_like_the_original_draws_it() {
    // trust / duty-support NPCs: ACT sends no job, and there is no "(owner)" in the name
    assert!(matches!(sorted("Aw'aern", ""), Participant::Npc(_)));
    assert!(matches!(sorted("Aw'zdei", ""), Participant::Npc(_)));
    // ... but a name with an owner is a pet, and a known pet's name on its own stays as it was
    assert!(sorted("Aw'aern (Player)", "").is_pet());
    assert!(matches!(sorted("Eos", ""), Participant::Unclassified(_)));
}

#[test]
fn owner_is_needed_for_pets_and_chocobos() {
    assert!(matches!(sorted("Eos", ""), Participant::Unclassified(_)));
    assert!(matches!(sorted("Boco", "0"), Participant::Unclassified(_)));
}

#[test]
fn the_gauge_is_recognised_by_its_name_in_the_originals_languages_and_is_not_an_npc() {
    for name in ["Limit Break", "リミットブレイク", "Transcendance", "Limitrausch", "极限技", "리미트 브레이크"] {
        assert!(matches!(sorted(name, ""), Participant::LimitBreak(_)), "{name}");
    }
}

#[test]
fn only_pets_have_a_pet_owner() {
    assert_eq!(sorted("Eos (Future Fade)", "").pet_owner(), Some("Future Fade"));
    assert_eq!(sorted("Boco (Future Fade)", "0").pet_owner(), None);
    assert_eq!(sorted("YOU", "Sch").pet_owner(), None);
}
