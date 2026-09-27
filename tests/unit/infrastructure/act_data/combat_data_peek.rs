// //! Messages written the way OverlayPlugin's WebSocket sends them: compact, keys in the order
// //! type, Encounter, Combatant, isActive.
//
// use crate::infrastructure::act::data::{OverlayMessage, ParseOptions};
// use super::*;
//
// const BEASTMASTER: &str = include_str!("samples/beastmaster.compact.json");
// const ZURVAN: &str = include_str!("samples/zurvan.compact.json");
//
// /// After leaving a trial: nothing to show, but ACT still says active.
// const IDLE_AFTER_TRIAL: &str = r#"{"type":"CombatData","Encounter":{"title":"Encounter","duration":"00:00","DURATION":"0","damage":"0","encdps":"0.00","CurrentZoneName":"Mor Dhona"},"Combatant":{},"isActive":"true"}"#;
//
// fn peek(message: &str) -> Option<CombatDataPeek> {
//     peek_combat_data(message)
// }
//
// #[test]
// fn idle_message_after_a_trial() {
//     assert_eq!(peek(IDLE_AFTER_TRIAL), Some(CombatDataPeek { is_active: true, has_combatants: false }));
// }
//
// #[test]
// fn real_fights() {
//     assert_eq!(peek(BEASTMASTER), Some(CombatDataPeek { is_active: true, has_combatants: true }));
//     assert_eq!(peek(ZURVAN), Some(CombatDataPeek { is_active: false, has_combatants: true }));
// }
//
// #[test]
// fn empty_and_inactive() {
//     let message = IDLE_AFTER_TRIAL.replace(r#""isActive":"true""#, r#""isActive":"false""#);
//     assert_eq!(peek(&message), Some(CombatDataPeek { is_active: false, has_combatants: false }));
// }
//
// #[test]
// fn peek_agrees_with_a_full_parse() {
//     for message in [BEASTMASTER, ZURVAN, IDLE_AFTER_TRIAL] {
//         let peeked = peek(message).unwrap();
//         let OverlayMessage::CombatData(parsed) = OverlayMessage::parse(message, ParseOptions::default()).unwrap()
//         else {
//             panic!("expected combat data")
//         };
//         assert_eq!(peeked.is_active, parsed.is_encounter_active);
//         assert_eq!(peeked.has_combatants, !parsed.combatants.is_empty());
//     }
// }
//
// /// Anything not in OverlayPlugin's exact format must fall back to a full parse, never skip.
// #[test]
// fn unknown_shapes_are_none() {
//     let pretty = include_str!("samples/beastmaster.json");
//     let boolean_active = IDLE_AFTER_TRIAL.replace(r#""isActive":"true""#, r#""isActive":true"#);
//     for message in [pretty, boolean_active.as_str(), "", "{}"] {
//         assert_eq!(peek(message), None, "{message:.60}");
//     }
// }
//
// /// Only combat data has an `isActive` key, so other messages never peek as combat data,
// /// even when their text mentions it: quotes inside text are escaped.
// #[test]
// fn other_message_types_are_none() {
//     let player = r#"{"type":"ChangePrimaryPlayer","charID":1,"charName":"Future Fade"}"#;
//     let log_line = r#"{"type":"LogLine","rawLine":"say \"Combatant\":{}, ,\"isActive\":\"true\"}"}"#;
//     assert!(serde_json::from_str::<serde_json::Value>(log_line).is_ok(), "the test message must be valid JSON");
//     for message in [player, log_line] {
//         assert_eq!(peek(message), None, "{message}");
//     }
// }
//
// /// Text inside a JSON string has its quotes escaped, so it cannot look like the real key.
// #[test]
// fn look_alike_text_in_a_name_is_not_the_key() {
//     let message = r#"{"type":"CombatData","Encounter":{"title":"\"Combatant\":{},"},"Combatant":{"YOU":{"name":"YOU"}},"isActive":"true"}"#;
//     assert_eq!(peek(message), Some(CombatDataPeek { is_active: true, has_combatants: true }));
// }
