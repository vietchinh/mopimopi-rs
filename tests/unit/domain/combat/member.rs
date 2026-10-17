use super::*;

fn member(name: &str, job: &str) -> Member {
    Member::sort(Figures { name: name.into(), job: job.to_string().into(), ..Figures::default() })
}

#[test]
fn base_classes_take_their_advanced_jobs_colour_and_the_limit_break_is_lmb() {
    assert_eq!(member("A", "Gla").class_code(), "PLD");
    assert_eq!(member("A", "Drk").class_code(), "DRK");
    assert_eq!(member("Limit Break", "Limit Break").class_code(), "LMB");
    assert_eq!(member("Chocobo (YOU)", "0").class_code(), "CBO");
}

#[test]
fn a_scholars_pet_counts_as_a_healer_and_a_dark_knights_as_a_tank() {
    assert!(member("Eos (YOU)", "").is_healer());
    assert!(member("Esteem (YOU)", "").is_tank());
    assert!(!member("Carbuncle (YOU)", "").is_healer());
}

#[test]
fn a_jobless_unowned_row_is_an_npc_but_the_gauge_is_not() {
    assert!(member("Aw'aern", "").is_npc());
    assert!(!member("Limit Break", "").is_npc(), "named like the gauge, no job");
    assert!(!member("Limit Break", "Limit Break").is_npc());
    assert!(!member("Alice", "Drk").is_npc());
    assert!(!member("Eos (YOU)", "").is_npc(), "a pet is not an NPC");
}

#[test]
fn every_row_is_sorted_into_exactly_one_kind() {
    assert!(matches!(member("Alice", "Drk"), Member::Player(_)));
    assert!(matches!(member("YOU", "Sch"), Member::Player(_)));
    assert!(matches!(member("Aw'aern", ""), Member::Npc(_)));
    assert!(matches!(member("Limit Break", "Limit Break"), Member::LimitBreak(_)));
    assert!(matches!(member("Limit Break", ""), Member::LimitBreak(_)));
    assert!(matches!(member("Eos (Alice)", ""), Member::Companion(PlayerCompanion { kind: CompanionKind::Pet(_), .. })));
    assert!(matches!(member("Boco (Alice)", "0"), Member::Companion(PlayerCompanion { kind: CompanionKind::Chocobo, .. })));
    assert!(matches!(member("Eos", ""), Member::Unclassified(_)), "a known pet name with no owner");
}

#[test]
fn a_companion_knows_its_owner() {
    match member("Eos (Future Fade)", "") {
        Member::Companion(companion) => assert_eq!(companion.owner, "Future Fade"),
        other => panic!("expected a companion, got {other:?}"),
    }
}
