use super::*;

fn stats(name: &str, job: &str) -> Stats {
    Stats { name: name.into(), job: job.to_string().into(), ..Stats::default() }
}

#[test]
fn base_classes_take_their_advanced_jobs_colour_and_the_limit_break_is_lmb() {
    assert_eq!(Participant::player(stats("A", "Gla")).class_code(), "PLD");
    assert_eq!(Participant::player(stats("A", "Drk")).class_code(), "DRK");
    assert_eq!(Participant::limit_break(stats("Limit Break", "Limit Break")).class_code(), "LMB");
    assert_eq!(Participant::chocobo(stats("Chocobo (YOU)", "0"), "YOU").class_code(), "CBO");
}

#[test]
fn a_scholars_pet_counts_as_a_healer_and_a_dark_knights_as_a_tank() {
    assert!(Participant::pet(stats("Eos (YOU)", ""), "YOU", PetJob::Scholar).is_healer());
    assert!(Participant::pet(stats("Esteem (YOU)", ""), "YOU", PetJob::DarkKnight).is_tank());
    assert!(!Participant::pet(stats("Carbuncle (YOU)", ""), "YOU", PetJob::Summoner).is_healer());
}

#[test]
fn an_npc_looks_like_the_gauge_but_is_not_the_gauge() {
    let npc = Participant::npc(stats("Aw'aern", ""));
    assert!(npc.is_npc() && npc.looks_like_limit_break());
    let gauge = Participant::limit_break(stats("Limit Break", "Limit Break"));
    assert!(!gauge.is_npc() && gauge.looks_like_limit_break());
    assert!(!Participant::player(stats("Alice", "Drk")).looks_like_limit_break());
}

#[test]
fn a_companion_knows_its_owner_and_a_pet_but_not_a_chocobo_is_folded() {
    let eos = Participant::pet(stats("Eos (Future Fade)", ""), "Future Fade", PetJob::Scholar);
    assert_eq!(eos.pet_owner(), Some("Future Fade"));
    assert_eq!(eos.companion_of(), Some(("Future Fade", CompanionKind::Pet(PetJob::Scholar))));
    let boco = Participant::chocobo(stats("Boco (Future Fade)", "0"), "Future Fade");
    assert_eq!(boco.pet_owner(), None);
    assert!(boco.is_chocobo());
}

#[test]
fn an_orphaned_pet_demoted_to_unclassified_still_knows_what_it_was() {
    let orphan = Participant::pet(stats("Eos (Bob)", ""), "Bob", PetJob::Scholar).demoted();
    assert!(matches!(orphan, Participant::Unclassified(_)));
    assert_eq!(orphan.pet_owner(), Some("Bob"));
    assert!(orphan.is_healer());
}

#[test]
fn whose_a_participant_is() {
    assert_eq!(Participant::you(stats("YOU", "Sch")).ownership(), Ownership::You);
    assert_eq!(Participant::pet(stats("Eos (YOU)", ""), "YOU", PetJob::Scholar).ownership(), Ownership::YourCompanion);
    assert_eq!(Participant::chocobo(stats("Boco (YOU)", "0"), "YOU").ownership(), Ownership::YourCompanion);
    assert_eq!(Participant::pet(stats("Eos (Alice)", ""), "Alice", PetJob::Scholar).ownership(), Ownership::Others);
    assert_eq!(Participant::player(stats("YOUNG", "Drk")).ownership(), Ownership::Others, "only the local player is you, whatever a name contains");
    assert!(Participant::pet(stats("Eos (YOU)", ""), "YOU", PetJob::Scholar).is_your_pet());
    assert!(!Participant::chocobo(stats("Boco (YOU)", "0"), "YOU").is_your_pet());
}
