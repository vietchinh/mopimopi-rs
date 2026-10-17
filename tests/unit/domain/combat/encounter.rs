use super::*;

fn stats(name: &str, job: &str, damage: f64) -> Stats {
    Stats { name: name.into(), job: job.to_string().into(), damage, duration_seconds: 50.0, ..Stats::default() }
}

fn participant(name: &str, job: &str, damage: f64) -> Participant {
    Participant::sort(stats(name, job, damage))
}

fn facts(duration_seconds: f64) -> EncounterFacts {
    EncounterFacts { title: "Zurvan".into(), duration_seconds, total_damage: 1000.0, total_healed: 200.0, ..EncounterFacts::default() }
}

#[test]
fn encounter_rate_uses_the_encounter_duration_and_personal_rate_the_members_own_time() {
    let alice = participant("Alice", "Drk", 500.0);
    let alice_stats = stats("Alice", "Drk", 500.0);
    let encounter = Encounter::from_stats(facts(100.0), vec![alice_stats.clone()], true);
    assert_eq!(encounter.damage_per_second_of(&alice).value(), 5.0);
    assert_eq!(alice.personal_damage_per_second().value(), 10.0);
}

#[test]
fn a_zero_encounter_duration_gives_infinity_and_a_zero_personal_time_divides_by_one() {
    let mut alice_stats = stats("Alice", "Drk", 500.0);
    alice_stats.duration_seconds = 0.0;
    let alice = Participant::sort(alice_stats.clone());
    let encounter = Encounter::from_stats(facts(0.0), vec![alice_stats.clone()], true);
    assert_eq!(encounter.damage_per_second_of(&alice).value(), f64::INFINITY);
    assert_eq!(alice.personal_damage_per_second().value(), 500.0);
}

#[test]
fn encdps_keeps_falling_while_damage_is_flat_and_the_fight_goes_on() {
    let alice = participant("Alice", "Drk", 1000.0);
    let alice_stats = stats("Alice", "Drk", 1000.0);
    let earlier = Encounter::from_stats(facts(10.0), vec![alice_stats.clone()], true);
    let later = Encounter::from_stats(facts(20.0), vec![alice_stats.clone()], true);
    assert_eq!(earlier.damage_per_second_of(&alice).value(), 100.0);
    assert_eq!(later.damage_per_second_of(&alice).value(), 50.0);
}

#[test]
fn rows_are_sorted_by_damage_with_ties_in_arrival_order() {
    let encounter = Encounter::from_stats(facts(10.0), vec![stats("B", "Drk", 5.0), stats("A", "Drk", 9.0), stats("C", "Drk", 5.0)], true);
    let names: Vec<_> = encounter.rows(PetView::Separate).into_iter().map(|row| row.name.clone()).collect();
    assert_eq!(names, ["A", "B", "C"]);
}

#[test]
fn folding_is_a_view_and_leaves_the_encounter_unchanged() {
    let participants = vec![stats("YOU", "Sch", 100.0), stats("Eos (Future Fade)", "", 20.0)];
    let encounter = Encounter::from_stats(facts(10.0), participants, true);
    let before = encounter.clone();
    let separate = encounter.rows(PetView::Separate);
    assert_eq!(separate.len(), 2);
    let folded = encounter.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") });
    assert_eq!(folded.len(), 1);
    assert_eq!(folded[0].damage, 120.0);
    assert_eq!(folded[0].pet_damage, 20.0);
    assert_eq!(encounter, before, "asking for a view changes nothing");
    assert_eq!(encounter.you().unwrap().damage, 100.0);
}

#[test]
fn a_pet_whose_owner_is_absent_stays_its_own_row_and_is_reported_as_orphaned() {
    let encounter = Encounter::from_stats(facts(10.0), vec![stats("Alice", "Drk", 10.0), stats("Eos (Bob)", "", 5.0)], true);
    let folded = encounter.rows(PetView::FoldedIntoOwners { local_player_name: None });
    assert_eq!(folded.len(), 2);
    assert_eq!(encounter.orphaned_pets().len(), 1);
}

#[test]
fn a_fight_ends_when_an_inactive_message_follows_an_active_one() {
    let over = Encounter::from_stats(facts(10.0), vec![], false);
    assert_eq!(over.ended_since(true).map(|event| event.key), Some(over.key()));
    assert_eq!(over.ended_since(false), None, "idle after idle is not an ending");
    assert_eq!(Encounter::from_stats(facts(10.0), vec![], true).ended_since(true), None);
}

#[test]
fn the_key_is_the_title_and_the_totals() {
    assert_eq!(Encounter::from_stats(facts(10.0), vec![], false).key().as_str(), "Zurvan1000200");
}

#[test]
fn building_an_encounter_with_a_companion_whose_owner_is_absent_is_refused() {
    let broken = vec![participant("Alice", "Drk", 10.0), participant("Eos (Bob)", "", 5.0)];
    assert_eq!(
        Encounter::new(facts(10.0), broken, true),
        Err(EncounterError::CompanionWithoutOwner { companion: "Eos (Bob)".into(), owner: "Bob".into() })
    );
}

#[test]
fn building_an_encounter_with_two_limit_breaks_is_refused() {
    let broken = vec![participant("Limit Break", "Limit Break", 0.0), participant("Limit Break", "", 0.0)];
    assert_eq!(Encounter::new(facts(10.0), broken, true), Err(EncounterError::SeveralLimitBreaks));
}

#[test]
fn a_companion_of_the_local_player_is_accepted_because_the_local_player_is_listed_as_you() {
    let ok = vec![participant("YOU", "Sch", 10.0), participant("Eos (Future Fade)", "", 5.0)];
    assert!(Encounter::new(facts(10.0), ok, true).is_ok());
}

#[test]
fn acts_odd_rows_are_placed_as_unclassified_instead_of_being_lost() {
    let encounter = Encounter::from_stats(
        facts(10.0),
        vec![stats("Alice", "Drk", 10.0), stats("Eos (Bob)", "", 5.0), stats("Limit Break", "Limit Break", 0.0), stats("Limit Break", "", 0.0)],
        true,
    );
    assert_eq!(encounter.participants().len(), 4);
    assert_eq!(encounter.players().count(), 1);
    assert_eq!(encounter.companions().count(), 0);
    assert!(encounter.limit_break().is_some());
    let unclassified = encounter.participants().iter().filter(|m| matches!(m, Participant::Unclassified(_))).count();
    assert_eq!(unclassified, 2);
    assert!(matches!(encounter.participants()[1].kind(), ParticipantKind::Pet { .. }), "an orphaned pet still looks like a pet");
}

#[test]
fn members_are_sorted_into_their_kinds() {
    let encounter = Encounter::from_stats(
        facts(10.0),
        vec![stats("Alice", "Drk", 10.0), stats("Aw'aern", "", 1.0), stats("Eos (Alice)", "", 5.0), stats("Limit Break", "Limit Break", 0.0)],
        true,
    );
    assert_eq!((encounter.players().count(), encounter.npcs().count(), encounter.companions().count()), (1, 1, 1));
    assert_eq!(encounter.limit_break().map(|m| m.name.as_str()), Some("Limit Break"));
}

#[test]
fn the_local_players_standing_is_ranked_by_damage_and_by_healing() {
    let mut healer = stats("YOU", "Sch", 300.0);
    healer.healed = 900.0;
    let mut tank = stats("Tank", "Pld", 500.0);
    tank.healed = 100.0;
    let encounter = Encounter::from_stats(facts(10.0), vec![healer, tank], true);
    let standing = encounter.local_standing(PetView::Separate).unwrap();
    assert_eq!((standing.damage_rank, standing.healing_rank, standing.of), (2, 1, 2));
    assert_eq!(standing.damage_per_second.value(), 30.0);
    assert_eq!(standing.healing_per_second.value(), 90.0);
}

#[test]
fn there_is_no_standing_without_the_local_player() {
    let encounter = Encounter::from_stats(facts(10.0), vec![stats("Alice", "Drk", 1.0)], true);
    assert_eq!(encounter.local_standing(PetView::Separate), None);
}

#[test]
fn the_standing_counts_the_local_players_folded_pets() {
    let encounter = Encounter::from_stats(facts(10.0), vec![stats("YOU", "Sch", 100.0), stats("Eos (Future Fade)", "", 100.0)], true);
    let folded = encounter.local_standing(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") }).unwrap();
    assert_eq!((folded.of, folded.damage_per_second.value()), (1, 20.0));
    assert_eq!(encounter.local_standing(PetView::Separate).unwrap().damage_per_second.value(), 10.0);
}

#[test]
fn you_are_one_of_the_participants_and_the_other_players_are_not_you() {
    let encounter = Encounter::from_stats(facts(10.0), vec![stats("YOU", "Sch", 100.0), stats("Alice", "Drk", 50.0)], true);
    assert!(matches!(encounter.you(), Some(Participant::You(_))));
    assert_eq!(encounter.players().count(), 2, "players() includes you");
    assert!(matches!(encounter.participants()[1], Participant::Player(_)));
}
