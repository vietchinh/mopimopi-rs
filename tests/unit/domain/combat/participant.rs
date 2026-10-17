use super::*;

fn participant(name: &str, job: &str) -> Participant {
    Participant::sort(Stats { name: name.into(), job: job.to_string().into(), ..Stats::default() })
}

#[test]
fn base_classes_take_their_advanced_jobs_colour_and_the_limit_break_is_lmb() {
    assert_eq!(participant("A", "Gla").class_code(), "PLD");
    assert_eq!(participant("A", "Drk").class_code(), "DRK");
    assert_eq!(participant("Limit Break", "Limit Break").class_code(), "LMB");
    assert_eq!(participant("Chocobo (YOU)", "0").class_code(), "CBO");
}

#[test]
fn a_scholars_pet_counts_as_a_healer_and_a_dark_knights_as_a_tank() {
    assert!(participant("Eos (YOU)", "").is_healer());
    assert!(participant("Esteem (YOU)", "").is_tank());
    assert!(!participant("Carbuncle (YOU)", "").is_healer());
}

#[test]
fn a_jobless_unowned_row_is_an_npc_but_the_gauge_is_not() {
    assert!(participant("Aw'aern", "").is_npc());
    assert!(!participant("Limit Break", "").is_npc(), "named like the gauge, no job");
    assert!(!participant("Limit Break", "Limit Break").is_npc());
    assert!(!participant("Alice", "Drk").is_npc());
    assert!(!participant("Eos (YOU)", "").is_npc(), "a pet is not an NPC");
}

#[test]
fn every_row_is_sorted_into_exactly_one_kind() {
    assert!(matches!(participant("Alice", "Drk"), Participant::Player(_)));
    assert!(matches!(participant("YOU", "Sch"), Participant::You(_)));
    assert!(matches!(participant("YOU", ""), Participant::You(_)), "ACT's own name for the local player decides, whatever the job text");
    assert!(matches!(participant("Aw'aern", ""), Participant::Npc(_)));
    assert!(matches!(participant("Limit Break", "Limit Break"), Participant::LimitBreak(_)));
    assert!(matches!(participant("Limit Break", ""), Participant::LimitBreak(_)));
    assert!(matches!(participant("Eos (Alice)", ""), Participant::Companion(PlayerCompanion { kind: CompanionKind::Pet(_), .. })));
    assert!(matches!(participant("Boco (Alice)", "0"), Participant::Companion(PlayerCompanion { kind: CompanionKind::Chocobo, .. })));
    assert!(matches!(participant("Eos", ""), Participant::Unclassified(_)), "a known pet name with no owner");
}

#[test]
fn a_companion_knows_its_owner() {
    match participant("Eos (Future Fade)", "") {
        Participant::Companion(companion) => assert_eq!(companion.owner, "Future Fade"),
        other => panic!("expected a companion, got {other:?}"),
    }
}

#[test]
fn the_gauge_is_recognised_by_its_name_in_the_originals_languages_and_is_not_an_npc() {
    for name in ["Limit Break", "リミットブレイク", "Transcendance", "Limitrausch", "极限技", "리미트 브레이크"] {
        assert!(matches!(participant(name, ""), Participant::LimitBreak(_)), "{name}");
        assert!(!participant(name, "").is_npc(), "{name}");
    }
}
