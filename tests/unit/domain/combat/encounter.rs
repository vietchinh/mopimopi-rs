use super::*;

fn member(name: &str, job: &str, damage: f64) -> Member {
    Member { name: name.into(), job_text: job.into(), damage, duration_seconds: 50.0, ..Member::default() }
}

fn facts(duration_seconds: f64) -> EncounterFacts {
    EncounterFacts { title: "Zurvan".into(), duration_seconds, total_damage: 1000.0, total_healed: 200.0, ..EncounterFacts::default() }
}

#[test]
fn encounter_rate_uses_the_encounter_duration_and_personal_rate_the_members_own_time() {
    let alice = member("Alice", "Drk", 500.0);
    let encounter = Encounter::new(facts(100.0), vec![alice.clone()], true);
    assert_eq!(encounter.damage_per_second_of(&alice).value(), 5.0);
    assert_eq!(alice.personal_damage_per_second().value(), 10.0);
}

#[test]
fn a_zero_encounter_duration_gives_infinity_and_a_zero_personal_time_divides_by_one() {
    let mut alice = member("Alice", "Drk", 500.0);
    alice.duration_seconds = 0.0;
    let encounter = Encounter::new(facts(0.0), vec![alice.clone()], true);
    assert_eq!(encounter.damage_per_second_of(&alice).value(), f64::INFINITY);
    assert_eq!(alice.personal_damage_per_second().value(), 500.0);
}

#[test]
fn encdps_keeps_falling_while_damage_is_flat_and_the_fight_goes_on() {
    let alice = member("Alice", "Drk", 1000.0);
    let earlier = Encounter::new(facts(10.0), vec![alice.clone()], true);
    let later = Encounter::new(facts(20.0), vec![alice.clone()], true);
    assert_eq!(earlier.damage_per_second_of(&alice).value(), 100.0);
    assert_eq!(later.damage_per_second_of(&alice).value(), 50.0);
}

#[test]
fn rows_are_sorted_by_damage_with_ties_in_arrival_order() {
    let encounter = Encounter::new(facts(10.0), vec![member("B", "Drk", 5.0), member("A", "Drk", 9.0), member("C", "Drk", 5.0)], true);
    let names: Vec<_> = encounter.rows(PetView::Separate).into_iter().map(|row| row.name).collect();
    assert_eq!(names, ["A", "B", "C"]);
}

#[test]
fn folding_is_a_view_and_leaves_the_encounter_unchanged() {
    let members = vec![member("YOU", "Sch", 100.0), member("Eos (Future Fade)", "", 20.0)];
    let encounter = Encounter::new(facts(10.0), members, true);
    let before = encounter.clone();
    let separate = encounter.rows(PetView::Separate);
    assert_eq!(separate.len(), 2);
    let folded = encounter.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") });
    assert_eq!(folded.len(), 1);
    assert_eq!(folded[0].damage, 120.0);
    assert_eq!(folded[0].pet_damage, 20.0);
    assert_eq!(encounter, before, "asking for a view changes nothing");
    assert_eq!(encounter.local_player().unwrap().damage, 100.0);
}

#[test]
fn a_pet_whose_owner_is_absent_stays_its_own_row_and_is_reported_as_orphaned() {
    let encounter = Encounter::new(facts(10.0), vec![member("Alice", "Drk", 10.0), member("Eos (Bob)", "", 5.0)], true);
    let folded = encounter.rows(PetView::FoldedIntoOwners { local_player_name: None });
    assert_eq!(folded.len(), 2);
    assert_eq!(encounter.orphaned_pets().len(), 1);
}

#[test]
fn a_fight_ends_when_an_inactive_message_follows_an_active_one() {
    let over = Encounter::new(facts(10.0), vec![], false);
    assert_eq!(over.ended_since(true).map(|event| event.key), Some(over.key()));
    assert_eq!(over.ended_since(false), None, "idle after idle is not an ending");
    assert_eq!(Encounter::new(facts(10.0), vec![], true).ended_since(true), None);
}

#[test]
fn the_key_is_the_title_and_the_totals() {
    assert_eq!(Encounter::new(facts(10.0), vec![], false).key().as_str(), "Zurvan1000200");
}
