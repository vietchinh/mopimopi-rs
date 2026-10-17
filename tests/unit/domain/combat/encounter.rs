use super::*;
use crate::domain::combat::{Amount, PetJob, Phase, Stats};

fn stats(name: &str, job: &str, damage: f64) -> Stats {
    Stats { name: name.into(), job: job.to_string().into(), damage: Amount::new(damage), duration_seconds: 50.0, ..Stats::default() }
}

// The domain receives participants already placed; these build them the way the ACT edge would.
fn you(damage: f64) -> Participant {
    Participant::you(stats("YOU", "Sch", damage))
}

fn player(name: &str, damage: f64) -> Participant {
    Participant::player(stats(name, "Drk", damage))
}

fn pet(name: &str, owner: &str, damage: f64) -> Participant {
    Participant::pet(stats(&format!("{name} ({owner})"), "", damage), owner, PetJob::Scholar)
}

fn gauge() -> Participant {
    Participant::limit_break(stats("Limit Break", "Limit Break", 0.0))
}

fn count(encounter: &Encounter, is_kind: impl Fn(&Participant) -> bool) -> usize {
    encounter.participants().iter().filter(|participant| is_kind(participant)).count()
}

fn facts(duration_seconds: f64) -> EncounterFacts {
    EncounterFacts { title: "Zurvan".into(), duration_seconds, total_damage: 1000.0, total_healed: 200.0, ..EncounterFacts::default() }
}

#[test]
fn encounter_rate_uses_the_encounter_duration_and_personal_rate_the_members_own_time() {
    let alice = player("Alice", 500.0);
    let encounter = Encounter::from_participants(facts(100.0), vec![alice.clone()], Phase::Running);
    assert_eq!(encounter.damage_per_second_of(&alice).value(), 5.0);
    assert_eq!(alice.personal_damage_per_second().value(), 10.0);
}

#[test]
fn a_zero_encounter_duration_gives_infinity_and_a_zero_personal_time_divides_by_one() {
    let mut alice = player("Alice", 500.0);
    alice.duration_seconds = 0.0;
    let encounter = Encounter::from_participants(facts(0.0), vec![alice.clone()], Phase::Running);
    assert_eq!(encounter.damage_per_second_of(&alice).value(), f64::INFINITY);
    assert_eq!(alice.personal_damage_per_second().value(), 500.0);
}

#[test]
fn encdps_keeps_falling_while_damage_is_flat_and_the_fight_goes_on() {
    let alice = player("Alice", 1000.0);
    let earlier = Encounter::from_participants(facts(10.0), vec![alice.clone()], Phase::Running);
    let later = Encounter::from_participants(facts(20.0), vec![alice.clone()], Phase::Running);
    assert_eq!(earlier.damage_per_second_of(&alice).value(), 100.0);
    assert_eq!(later.damage_per_second_of(&alice).value(), 50.0);
}

#[test]
fn rows_are_sorted_by_damage_with_ties_in_arrival_order() {
    let encounter = Encounter::from_participants(facts(10.0), vec![player("B", 5.0), player("A", 9.0), player("C", 5.0)], Phase::Running);
    let names: Vec<_> = encounter.rows(PetView::Separate).into_iter().map(|row| row.name.clone()).collect();
    assert_eq!(names, ["A", "B", "C"]);
}

#[test]
fn folding_is_a_view_and_leaves_the_encounter_unchanged() {
    let encounter = Encounter::from_participants(facts(10.0), vec![you(100.0), pet("Eos", "Future Fade", 20.0)], Phase::Running);
    let before = encounter.clone();
    let separate = encounter.rows(PetView::Separate);
    assert_eq!(separate.len(), 2);
    let folded = encounter.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") });
    assert_eq!(folded.len(), 1);
    assert_eq!(folded[0].damage, Amount::new(120.0));
    assert_eq!(folded[0].pet_damage, Amount::new(20.0));
    assert_eq!(encounter, before, "asking for a view changes nothing");
    assert_eq!(encounter.participants().iter().find(|p| p.is_you()).unwrap().damage, Amount::new(100.0));
}

#[test]
fn a_pet_whose_owner_is_absent_stays_its_own_row_() {
    let encounter = Encounter::from_participants(facts(10.0), vec![player("Alice", 10.0), pet("Eos", "Bob", 5.0)], Phase::Running);
    let folded = encounter.rows(PetView::FoldedIntoOwners { local_player_name: None });
    assert_eq!(folded.len(), 2);
    assert!(folded.iter().any(|row| row.is_pet()), "the pet is still a pet, shown on its own");
}

#[test]
fn a_fight_ends_when_an_inactive_message_follows_an_active_one() {
    let over = Encounter::from_participants(facts(10.0), vec![], Phase::Over);
    assert_eq!(over.ended_since(Phase::Running).map(|event| event.key), Some(over.key()));
    assert_eq!(over.ended_since(Phase::Over), None, "idle after idle is not an ending");
    assert_eq!(Encounter::from_participants(facts(10.0), vec![], Phase::Running).ended_since(Phase::Running), None);
}

#[test]
fn the_key_is_the_title_and_the_totals() {
    assert_eq!(Encounter::from_participants(facts(10.0), vec![], Phase::Over).key().as_str(), "Zurvan1000200");
}

#[test]
fn building_an_encounter_with_a_companion_whose_owner_is_absent_is_refused() {
    let broken = vec![player("Alice", 10.0), pet("Eos", "Bob", 5.0)];
    assert_eq!(
        Encounter::new(facts(10.0), broken, Phase::Running),
        Err(EncounterError::CompanionWithoutOwner { companion: "Eos (Bob)".into(), owner: "Bob".into() })
    );
}

#[test]
fn building_an_encounter_with_two_limit_breaks_is_refused() {
    let broken = vec![gauge(), gauge()];
    assert_eq!(Encounter::new(facts(10.0), broken, Phase::Running), Err(EncounterError::SeveralLimitBreaks));
}

#[test]
fn a_companion_of_the_local_player_is_accepted_because_the_local_player_is_listed_as_you() {
    let ok = vec![you(10.0), pet("Eos", "Future Fade", 5.0)];
    assert!(Encounter::new(facts(10.0), ok, Phase::Running).is_ok());
}

#[test]
fn acts_odd_rows_are_placed_as_unclassified_instead_of_being_lost() {
    let encounter = Encounter::from_participants(facts(10.0), vec![player("Alice", 10.0), pet("Eos", "Bob", 5.0), gauge(), gauge()], Phase::Running);
    assert_eq!(encounter.participants().len(), 4);
    assert_eq!(count(&encounter, |p| matches!(p, Participant::Player(_) | Participant::You(_))), 1);
    assert_eq!(count(&encounter, |p| matches!(p, Participant::Companion(_))), 0);
    assert_eq!(count(&encounter, |p| matches!(p, Participant::LimitBreak(_))), 1);
    let unclassified = encounter.participants().iter().filter(|m| matches!(m.as_ref(), Participant::Unclassified(_))).count();
    assert_eq!(unclassified, 2);
    assert!(encounter.participants()[1].is_pet(), "an orphaned pet still looks like a pet");
}

#[test]
fn members_are_sorted_into_their_kinds() {
    let encounter = Encounter::from_participants(
        facts(10.0),
        vec![player("Alice", 10.0), Participant::npc(stats("Aw'aern", "", 1.0)), pet("Eos", "Alice", 5.0), gauge()],
        Phase::Running,
    );
    assert_eq!((count(&encounter, |p| matches!(p, Participant::Player(_) | Participant::You(_))), count(&encounter, |p| matches!(p, Participant::Npc(_))), count(&encounter, |p| matches!(p, Participant::Companion(_)))), (1, 1, 1));
    assert_eq!(count(&encounter, |p| matches!(p, Participant::LimitBreak(_))), 1);
}

#[test]
fn the_local_players_standing_is_ranked_by_damage_and_by_healing() {
    let mut healer = you(300.0);
    healer.healed = Amount::new(900.0);
    let mut tank = player("Tank", 500.0);
    tank.healed = Amount::new(100.0);
    let encounter = Encounter::from_participants(facts(10.0), vec![healer, tank], Phase::Running);
    let standing = encounter.local_standing(PetView::Separate).unwrap();
    assert_eq!((standing.damage_rank, standing.healing_rank, standing.of), (2, 1, 2));
    assert_eq!(standing.damage_per_second.value(), 30.0);
    assert_eq!(standing.healing_per_second.value(), 90.0);
}

#[test]
fn there_is_no_standing_without_the_local_player() {
    let encounter = Encounter::from_participants(facts(10.0), vec![player("Alice", 1.0)], Phase::Running);
    assert_eq!(encounter.local_standing(PetView::Separate), None);
}

#[test]
fn the_standing_counts_the_local_players_folded_pets() {
    let encounter = Encounter::from_participants(facts(10.0), vec![you(100.0), pet("Eos", "Future Fade", 100.0)], Phase::Running);
    let folded = encounter.local_standing(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") }).unwrap();
    assert_eq!((folded.of, folded.damage_per_second.value()), (1, 20.0));
    assert_eq!(encounter.local_standing(PetView::Separate).unwrap().damage_per_second.value(), 10.0);
}

#[test]
fn you_are_one_of_the_participants_and_the_other_players_are_not_you() {
    let encounter = Encounter::from_participants(facts(10.0), vec![you(100.0), player("Alice", 50.0)], Phase::Running);
    assert_eq!(count(&encounter, |p| p.is_you()), 1);
    assert_eq!(count(&encounter, |p| matches!(p, Participant::Player(_) | Participant::You(_))), 2, "players() includes you");
    assert!(matches!(*encounter.participants()[1], Participant::Player(_)));
}
