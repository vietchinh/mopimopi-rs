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
    assert!(crate::domain::combat::Encounter::from(combat_data.clone()).rows(crate::domain::combat::PetView::Separate).iter().any(|row| row.is_you()));
}

#[test]
fn the_type_may_come_anywhere_in_the_message() {
    let combat = r#"{"Encounter":{"title":"Boss"},"Combatant":{},"isActive":"true","type":"CombatData"}"#;
    let message = OverlayMessage::parse(combat).unwrap();
    assert_eq!(message.combat_data().unwrap().encounter.title, "Boss");
    assert!(message.combat_data().unwrap().is_encounter_active);
    let player = OverlayMessage::parse(r#"{"charID":7,"type":"ChangePrimaryPlayer","charName":"Future Fade"}"#).unwrap();
    assert_eq!(player.change_primary_player().unwrap().char_name, "Future Fade");
    assert_eq!(OverlayMessage::parse(r#"{"zoneID":132,"type":"ChangeZone"}"#).unwrap().kind(), MessageType::NotSupported);
}

#[test]
fn a_message_without_a_type_is_an_error_and_so_is_trailing_text() {
    assert!(OverlayMessage::parse(r#"{"Combatant":{},"isActive":"true"}"#).is_err());
    assert!(OverlayMessage::parse("{}").is_err());
    assert!(OverlayMessage::parse(r#"{"type":"ChangeZone"} x"#).is_err());
    assert!(OverlayMessage::parse(r#"{"type":"CombatData","Combatant":{}} x"#).is_err());
}

#[test]
fn reading_in_place_gives_the_same_message_as_reading_after_a_buffer() {
    // The capture with its "type" moved to the end takes the slow route; both must give the same message.
    let text = include_str!("samples/beastmaster.json");
    let rest = text.trim().strip_prefix(r#"{"type":"CombatData","#).unwrap().strip_suffix('}').unwrap();
    let moved = format!("{{{rest},\"type\":\"CombatData\"}}");
    let fast = OverlayMessage::parse(text).unwrap();
    let slow = OverlayMessage::parse(&moved).unwrap();
    assert_eq!(fast, slow);
}
