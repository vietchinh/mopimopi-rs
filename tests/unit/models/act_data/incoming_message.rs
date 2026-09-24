//! Unit tests for `act_data::incoming_message`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn overlay_plugin_combat_data() {
    let json = r#"{"type":"CombatData","Encounter":{"title":"Boss"},"Combatant":{},"isActive":"true"}"#;
    match parse_incoming_message(json).unwrap() {
        Some(ActEvent::CombatData(message)) => {
            assert_eq!(message.encounter.title, "Boss");
            assert!(message.is_encounter_active);
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn overlay_plugin_player_name() {
    let json = r#"{"type":"ChangePrimaryPlayer","charName":"Future Fade"}"#;
    assert_eq!(parse_incoming_message(json).unwrap(), Some(ActEvent::LocalPlayerName("Future Fade".into())));
}

#[test]
fn mini_parse_broadcasts() {
    let combat = r#"{"type":"broadcast","msgtype":"CombatData","msg":{"Encounter":{"title":"T"},"Combatant":{}}}"#;
    assert!(matches!(parse_incoming_message(combat).unwrap(), Some(ActEvent::CombatData(_))));
    let name = r#"{"type":"broadcast","msgtype":"SendCharName","msg":{"charName":"Me"}}"#;
    assert_eq!(parse_incoming_message(name).unwrap(), Some(ActEvent::LocalPlayerName("Me".into())));
}

#[test]
fn unknown_messages_are_ignored_and_garbage_is_an_error() {
    assert_eq!(parse_incoming_message(r#"{"type":"Whatever","x":1}"#).unwrap(), None);
    assert_eq!(parse_incoming_message(r#"{"type":"broadcast","msgtype":"Other","msg":1}"#).unwrap(), None);
    assert!(parse_incoming_message("not json").is_err());
}
