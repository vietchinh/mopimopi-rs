use super::*;
use crate::domain::combat::EncounterFacts;

fn fight(title: &str, zone: &str, damage: f64) -> Rc<Encounter> {
    let facts = EncounterFacts { title: title.into(), zone_name: zone.into(), total_damage: damage, duration_seconds: 10.0, ..EncounterFacts::default() };
    Rc::new(Encounter::from_stats(facts, vec![], false))
}

#[test]
fn a_repeat_of_the_previous_fight_is_not_recorded_again() {
    let mut history = History::default();
    assert!(history.record(fight("Zurvan", "Bay", 100.0)).is_some());
    assert!(history.record(fight("Zurvan", "Bay", 100.0)).is_none());
    assert_eq!(history.entries().len(), 1);
}

#[test]
fn only_the_previous_fight_counts_as_a_repeat() {
    let mut history = History::default();
    history.record(fight("A", "Bay", 1.0));
    history.record(fight("B", "Bay", 2.0));
    assert!(history.record(fight("A", "Bay", 1.0)).is_some(), "A again, but B came in between");
    assert_eq!(history.entries().len(), 3);
}

#[test]
fn fights_are_numbered_within_their_zone_and_a_new_zone_starts_again() {
    let mut history = History::default();
    let numbers: Vec<usize> = [("A", "Bay"), ("B", "Bay"), ("C", "Bay"), ("D", "Tower"), ("E", "Tower"), ("F", "Bay")]
        .iter()
        .enumerate()
        .map(|(index, (title, zone))| history.record(fight(title, zone, index as f64)).unwrap().number_in_zone())
        .collect();
    assert_eq!(numbers, [1, 2, 3, 1, 2, 1]);
}

#[test]
fn newest_is_first_and_entries_can_be_found_by_key() {
    let mut history = History::default();
    history.record(fight("A", "Bay", 1.0));
    history.record(fight("B", "Bay", 2.0));
    assert_eq!(history.entries()[0].title(), "B");
    let key = EncounterKey::new("A", 1.0, 0.0);
    assert_eq!(history.find(&key).unwrap().title(), "A");
}

#[test]
fn an_encounter_without_a_target_is_called_no_data() {
    let mut history = History::default();
    history.record(fight("Encounter", "Bay", 0.0));
    assert_eq!(history.entries()[0].title(), "No Data");
}
