// //! End to end: JSON in, merged combatants out.
//
// use super::*;
// use crate::domain::combat::CombatantKind;
//
// const BEASTMASTER: &str = include_str!("samples/beastmaster.json");
// const ZURVAN: &str = include_str!("samples/zurvan.json");
//
// const SEPARATE: ParseOptions<'static> = ParseOptions { merge_pets_into_owner: false, local_player_name: None };
// const MERGE: ParseOptions<'static> = ParseOptions { merge_pets_into_owner: true, local_player_name: None };
//
// fn parse(json: &str, options: ParseOptions<'_>) -> CombatDataMessage {
//     match OverlayMessage::parse(json, options).unwrap() {
//         OverlayMessage::CombatData(combat_data) => combat_data,
//         other => panic!("expected combat data, got {other:?}"),
//     }
// }
//
// fn sorted_names(combat_data: &CombatDataMessage) -> Vec<&str> {
//     let mut names: Vec<&str> = combat_data.combatants.iter().map(|combatant| combatant.name.as_str()).collect();
//     names.sort();
//     names
// }
//
// #[test]
// fn beastmaster_pet_merges_into_you() {
//     assert_eq!(sorted_names(&parse(BEASTMASTER, SEPARATE)), ["Cu Sith (YOU)", "YOU"]);
//
//     let merged = parse(BEASTMASTER, MERGE);
//     assert_eq!(sorted_names(&merged), ["YOU"]);
//     let you = merged.combatant("YOU").unwrap();
//     assert_eq!(you.damage, 20517.0 + 6241.0);
//     assert_eq!(you.hits, 32.0 + 19.0);
//     assert_eq!(you.duration_seconds, 36.0); // the owner's, not the pet's
//     assert_eq!(you.strongest_hit_text, "Shieldsplitter-1819");
// }
//
// #[test]
// fn own_pet_named_after_real_character_merges_into_you() {
//     let json = r#"{"type":"CombatData","Encounter":{"title":"x"},"isActive":"true","Combatant":{
//         "YOU":{"name":"YOU","Job":"Sch","damage":"100","damagetaken":"10","healstaken":"5","DURATION":"30"},
//         "Eos (Future Fade)":{"name":"Eos (Future Fade)","Job":"","damage":"20","damagetaken":"3","healstaken":"2","DURATION":"40","healed":"500"},
//         "Boco (Future Fade)":{"name":"Boco (Future Fade)","Job":"0","damage":"7"}}}"#;
//
//     let with_name = ParseOptions { merge_pets_into_owner: true, local_player_name: Some("Future Fade") };
//     let merged = parse(json, with_name);
//     assert_eq!(sorted_names(&merged), ["Boco (Future Fade)", "YOU"]); // the chocobo stays its own row
//     let you = merged.combatant("YOU").unwrap();
//     assert_eq!(you.damage, 120.0);
//     assert_eq!(you.damage_taken, 13.0);
//     assert_eq!(you.heals_taken, 7.0);
//     assert_eq!(you.healed, 500.0);
//     assert_eq!(you.duration_seconds, 30.0);
//
//     // Without the name, the guess finds YOU as well: Future Fade is the only missing owner.
//     assert_eq!(sorted_names(&parse(json, MERGE)), ["Boco (Future Fade)", "YOU"]);
// }
//
// #[test]
// fn several_pets_add_up_in_one_owner() {
//     let json = r#"{"type":"CombatData","Encounter":{"title":"x"},"Combatant":{
//         "Kay Lionheart":{"name":"Kay Lionheart","Job":"Smn","damage":"100","MAXHIT":"50","maxhit":"Ruin-50"},
//         "Demi-Bahamut (Kay Lionheart)":{"name":"Demi-Bahamut (Kay Lionheart)","Job":"","damage":"30","MAXHIT":"80","maxhit":"Akh Morn-80"},
//         "Carbuncle (Kay Lionheart)":{"name":"Carbuncle (Kay Lionheart)","Job":"","damage":"5","MAXHIT":"5","maxhit":"Attack-5"}}}"#;
//
//     let merged = parse(json, MERGE);
//     assert_eq!(sorted_names(&merged), ["Kay Lionheart"]);
//     let owner = merged.combatant("Kay Lionheart").unwrap();
//     assert_eq!(owner.damage, 135.0);
//     assert_eq!(owner.strongest_hit_amount, 80.0);
//     assert_eq!(owner.strongest_hit_text, "Akh Morn-80");
// }
//
// #[test]
// fn zurvan_has_no_pets_to_merge() {
//     let separate = parse(ZURVAN, SEPARATE);
//     let merged = parse(ZURVAN, MERGE);
//     assert_eq!(sorted_names(&separate), sorted_names(&merged));
//     assert_eq!(merged.combatants.len(), 9);
//     assert_eq!(merged.combatant("Limit Break").unwrap().kind(), CombatantKind::LimitBreak);
// }
//
// #[test]
// fn bare_data_merges_too() {
//     assert_eq!(sorted_names(&parse_bare_combat_data(BEASTMASTER, MERGE).unwrap()), ["YOU"]);
//     assert_eq!(sorted_names(&parse_bare_combat_data(BEASTMASTER, SEPARATE).unwrap()).len(), 2);
// }
//
// #[test]
// fn player_name_message() {
//     let json = r#"{"type":"ChangePrimaryPlayer","charID":268618589,"charName":"Future Fade"}"#;
//     let message = OverlayMessage::parse(json, SEPARATE).unwrap();
//     assert_eq!(message.kind(), MessageType::ChangePrimaryPlayer);
//     let player = message.change_primary_player().unwrap();
//     assert_eq!(player.char_id, 268_618_589);
//     assert_eq!(player.char_name, "Future Fade");
//     assert!(message.combat_data().is_none());
// }
//
// #[test]
// fn other_types_and_miniparse_broadcasts_are_not_supported() {
//     let zone_change = r#"{"type":"ChangeZone","zoneID":1,"nested":{"a":[1,2]}}"#;
//     let broadcast = format!(r#"{{"type":"broadcast","msgtype":"CombatData","msg":{BEASTMASTER}}}"#);
//     assert_eq!(OverlayMessage::parse(zone_change, SEPARATE).unwrap(), OverlayMessage::NotSupported);
//     assert_eq!(OverlayMessage::parse(&broadcast, SEPARATE).unwrap(), OverlayMessage::NotSupported);
// }
//
// #[test]
// fn malformed_messages_are_errors() {
//     assert!(OverlayMessage::parse("{not json", SEPARATE).is_err());
//     assert!(OverlayMessage::parse("{}", SEPARATE).is_err()); // no "type"
//     assert!(OverlayMessage::parse(r#"{"type":"ChangePrimaryPlayer","charID":1}"#, SEPARATE).is_err()); // no "charName"
//     assert!(OverlayMessage::parse(r#"{"type":"CombatData","Combatant":{}}"#, SEPARATE).is_err()); // no "Encounter"
//     assert!(OverlayMessage::parse(r#"{"type":"ChangeZone"} trailing"#, SEPARATE).is_err());
// }
//
// #[test]
// fn type_does_not_have_to_come_first() {
//     let json = r#"{"charName":"Future Fade","charID":1,"type":"ChangePrimaryPlayer"}"#;
//     assert_eq!(OverlayMessage::parse(json, SEPARATE).unwrap().kind(), MessageType::ChangePrimaryPlayer);
// }
//
// #[test]
// fn plain_serde_json_works_without_options() {
//     let message: OverlayMessage = serde_json::from_str(BEASTMASTER).unwrap();
//     let OverlayMessage::CombatData(combat_data) = message else { panic!("expected combat data") };
//     assert_eq!(sorted_names(&combat_data), ["Cu Sith (YOU)", "YOU"]);
// }
//
// /// The way the app uses it: remember the name, pass it back in for the next parse.
// #[test]
// fn match_on_messages_like_the_app_does() {
//     let messages = [
//         r#"{"type":"ChangePrimaryPlayer","charID":1,"charName":"Future Fade"}"#,
//         r#"{"type":"CombatData","Encounter":{"title":"x"},"Combatant":{
//             "YOU":{"name":"YOU","Job":"Sch","damage":"100"},
//             "Eos (Future Fade)":{"name":"Eos (Future Fade)","Job":"","damage":"20"}}}"#,
//     ];
//
//     let mut local_player_name: Option<String> = None;
//     let mut latest_combat_data = None;
//     for json in messages {
//         let options = ParseOptions {
//             merge_pets_into_owner: true,
//             local_player_name: local_player_name.as_deref(),
//         };
//         match OverlayMessage::parse(json, options).unwrap() {
//             OverlayMessage::CombatData(combat_data) => latest_combat_data = Some(combat_data),
//             OverlayMessage::ChangePrimaryPlayer(player) => local_player_name = Some(player.char_name),
//             OverlayMessage::NotSupported => {}
//         }
//     }
//
//     let combat_data = latest_combat_data.unwrap();
//     assert_eq!(sorted_names(&combat_data), ["YOU"]);
//     assert_eq!(combat_data.combatant("YOU").unwrap().damage, 120.0);
// }
