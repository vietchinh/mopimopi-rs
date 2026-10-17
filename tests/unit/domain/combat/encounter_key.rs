use super::*;

#[test]
fn key_is_title_then_totals_with_no_separator() {
    assert_eq!(EncounterKey::new("Zurvan", 1_943_397.0, 660_442.0).as_str(), "Zurvan1943397660442");
}

#[test]
fn same_fight_gives_equal_keys_and_a_different_total_does_not() {
    assert_eq!(EncounterKey::new("A", 1.0, 2.0), EncounterKey::new("A", 1.0, 2.0));
    assert_ne!(EncounterKey::new("A", 1.0, 2.0), EncounterKey::new("A", 1.0, 3.0));
}
