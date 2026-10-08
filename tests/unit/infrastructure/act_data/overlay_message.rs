//! Unit tests for `infrastructure::act::data::overlay_message`. Compiled as a child module of
//! that file (so they can use its private items) but kept here, outside `src/`.

use super::*;

#[test]
fn parses_combat_data() {
    let text = r#"{"type":"CombatData","Encounter":{"title":"Boss"},"Combatant":{},"isActive":"true"}"#;
    let message = OverlayMessage::parse(text).unwrap();
    assert_eq!(message.kind(), MessageType::CombatData);
    assert_eq!(message.combat_data().unwrap().encounter.title, "Boss");
    assert!(message.change_primary_player().is_none());
}

#[test]
fn parses_change_primary_player() {
    let text = r#"{"type":"ChangePrimaryPlayer","charID":268938170,"charName":"Future Fade"}"#;
    let message = OverlayMessage::parse(text).unwrap();
    assert_eq!(message.kind(), MessageType::ChangePrimaryPlayer);
    let player = message.change_primary_player().unwrap();
    assert_eq!((player.char_id, player.char_name.as_str()), (268938170, "Future Fade"));
    assert!(message.combat_data().is_none());
}

#[test]
fn any_other_type_is_not_supported_including_the_legacy_broadcast_envelope() {
    for text in [
        r#"{"type":"ChangeZone","zoneID":132}"#,
        r#"{"type":"broadcast","msgtype":"CombatData","msg":{}}"#,
        r#"{"type":"LogLine","line":["..."]}"#,
    ] {
        let message = OverlayMessage::parse(text).unwrap();
        assert_eq!(message.kind(), MessageType::NotSupported, "{text:?}");
    }
}

#[test]
fn malformed_json_is_an_error() {
    assert!(OverlayMessage::parse("{not json").is_err());
}

#[test]
fn a_real_capture_parses() {
    let text = include_str!("samples/beastmaster.json");
    let message = OverlayMessage::parse(text).unwrap();
    let combat_data = message.combat_data().expect("the capture is a CombatData message");
    assert!(!combat_data.combatants.is_empty());
    assert!(combat_data.local_player().is_some());
}

#[test]
fn parse_options_default_to_no_merging() {
    let options = ParseOptions::default();
    assert!(!options.merge_pets_into_owner);
    assert_eq!(options.local_player_name, None);
}
