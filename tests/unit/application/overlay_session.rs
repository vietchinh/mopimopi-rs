use super::*;
use std::rc::Rc;

const PLAYER: &str = r#"{"type":"ChangePrimaryPlayer","charID":1,"charName":"Future Fade"}"#;
const COMBAT: &str = r#"{"type":"CombatData","Encounter":{"title":"x"},"Combatant":{
    "YOU":{"name":"YOU","Job":"Sch","damage":"100"},
    "Eos (Future Fade)":{"name":"Eos (Future Fade)","Job":"","damage":"20"}},"isActive":"true"}"#;
const ZONE_CHANGE: &str = r#"{"type":"ChangeZone","zoneID":1}"#;

fn names(snapshot: &CombatDataMessage) -> Vec<&str> {
    let mut names: Vec<&str> = snapshot.combatants.iter().map(|c| c.name.as_str()).collect();
    names.sort();
    names
}

#[test]
fn remembers_the_player_name_for_the_next_merge() {
    let mut session = OverlaySession::default();
    session.set_merge_pets_into_owner(true);

    assert!(session.receive(PLAYER).unwrap().is_none());
    assert_eq!(session.local_player_name(), Some("Future Fade"));

    let snapshot = session.receive(COMBAT).unwrap().unwrap();
    assert_eq!(names(&snapshot), ["YOU"]);
    assert_eq!(snapshot.combatant("YOU").unwrap().damage, 120.0);
}

#[test]
fn pets_stay_separate_unless_merging_is_on() {
    let mut session = OverlaySession::default();
    let snapshot = session.receive(COMBAT).unwrap().unwrap();
    assert_eq!(names(&snapshot), ["Eos (Future Fade)", "YOU"]);
}

#[test]
fn the_snapshot_is_shared_not_copied() {
    let mut session = OverlaySession::default();
    let returned = session.receive(COMBAT).unwrap().unwrap();
    let for_a_component = session.latest_combat_data().unwrap();

    assert!(Rc::ptr_eq(&returned, &for_a_component)); // the same records, not a copy
    assert_eq!(Rc::strong_count(&returned), 3); // the session, `returned`, the component
}

#[test]
fn a_new_message_is_a_new_snapshot() {
    let mut session = OverlaySession::default();
    let first = session.receive(COMBAT).unwrap().unwrap();
    let second = session.receive(COMBAT).unwrap().unwrap();

    assert!(!Rc::ptr_eq(&first, &second)); // "did it change?" is one pointer comparison
    assert_eq!(*first, *second); // same content, but a component holding `first` keeps it
    assert!(Rc::ptr_eq(&session.latest_combat_data().unwrap(), &second));
}

#[test]
fn other_messages_and_errors_leave_the_snapshot_alone() {
    let mut session = OverlaySession::default();
    let snapshot = session.receive(COMBAT).unwrap().unwrap();

    assert!(session.receive(ZONE_CHANGE).unwrap().is_none());
    assert!(session.receive("{not json").is_err());
    assert!(Rc::ptr_eq(&session.latest_combat_data().unwrap(), &snapshot));
}
