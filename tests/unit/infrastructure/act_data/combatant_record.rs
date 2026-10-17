// //! Healing, shares and rates on real records, checked against ACT's own (rounded) values.
//
// use crate::infrastructure::act_data::{CombatDataMessage, OverlayMessage, ParseOptions};
//
// const ZURVAN: &str = include_str!("samples/zurvan.json");
// const BEASTMASTER: &str = include_str!("samples/beastmaster.json");
//
// fn parse(json: &str, merge_pets_into_owner: bool) -> CombatDataMessage {
//     let options = ParseOptions { merge_pets_into_owner, local_player_name: None };
//     match OverlayMessage::parse(json, options).unwrap() {
//         OverlayMessage::CombatData(combat_data) => combat_data,
//         other => panic!("expected combat data, got {other:?}"),
//     }
// }
//
// /// How ACT shows rates such as crit %.
// fn rounded(value: Option<f64>) -> f64 {
//     value.expect("expected a percentage").round()
// }
//
// /// How ACT shows shares such as damage % and healed %.
// fn cut_off(value: Option<f64>) -> f64 {
//     value.expect("expected a percentage").trunc()
// }
//
// #[test]
// fn healing_numbers_match_act() {
//     let zurvan = parse(ZURVAN, false);
//     let hamtaro = zurvan.combatant("Hamtaro Ghrimtaro").unwrap();
//     let total_healed = zurvan.encounter.total_healed;
//
//     assert_eq!(hamtaro.healing().effective(), 145_993.0);
//     assert_eq!(rounded(hamtaro.healing().overheal_percent()), 26.0); // ACT: "26%"
//     assert_eq!(cut_off(hamtaro.healing_share_percent(total_healed)), 41.0); // ACT: "41%"
//     assert_eq!(rounded(hamtaro.critical_heal_percent()), 23.0); // ACT: "23%"
//     assert_eq!(hamtaro.heal_per_second, 870.28);
// }
//
// #[test]
// fn damage_share_matches_act() {
//     let zurvan = parse(ZURVAN, false);
//     let gwenn = zurvan.combatant("Gwenn Windspeaker").unwrap();
//     assert_eq!(cut_off(gwenn.damage_share_percent(zurvan.encounter.total_damage)), 17.0); // ACT: "17%"
// }
//
// #[test]
// fn combatant_rates_add_up_to_the_encounter() {
//     let zurvan = parse(ZURVAN, false);
//     let damage_per_second: f64 = zurvan.combatants.iter().map(|c| c.damage_per_second).sum();
//     let heal_per_second: f64 = zurvan.combatants.iter().map(|c| c.heal_per_second).sum();
//     assert!((damage_per_second - zurvan.encounter.damage_per_second).abs() < 0.05);
//     assert!((heal_per_second - zurvan.encounter.heal_per_second).abs() < 0.05);
//     assert_eq!(zurvan.encounter.damage_per_second, 6106.61); // exact, not the rounded "6107"
// }
//
// #[test]
// fn infinite_rates_stay_infinite() {
//     let json = r#"{"type":"CombatData","Encounter":{"encdps":"∞"},"Combatant":{
//         "YOU":{"name":"YOU","Job":"Sge","damage":"10","encdps":"Infinity","Last30DPS":"Infinity"}}}"#;
//     let combat_data = parse(json, false);
//     assert!(combat_data.encounter.damage_per_second.is_infinite());
//     assert!(combat_data.combatant("YOU").unwrap().damage_per_second.is_infinite());
//     assert_eq!(combat_data.combatant("YOU").unwrap().damage_per_second_last_30_seconds, 0.0); // not a rate field
// }
//
// #[test]
// fn pet_shares_are_remembered_for_the_bars() {
//     let separate = parse(BEASTMASTER, false);
//     assert_eq!(separate.combatant("YOU").unwrap().pet_damage, 0.0);
//
//     let merged = parse(BEASTMASTER, true);
//     let you = merged.combatant("YOU").unwrap();
//     assert_eq!(you.pet_damage, 6241.0);
//     assert_eq!(you.damage - you.pet_damage, 20517.0); // the player's own part
//     assert!((you.damage_per_second - 595.47).abs() < 0.005); // 456.58 + 138.89 = the encounter's 595.47
// }
//
// #[test]
// fn fairy_healing_share_for_the_bar() {
//     let json = r#"{"type":"CombatData","Encounter":{"healed":"1000"},"Combatant":{
//         "YOU":{"name":"YOU","Job":"Sch","healed":"600","overHeal":"100","damageShield":"200","enchps":"6"},
//         "Eos (Future Fade)":{"name":"Eos (Future Fade)","Job":"","healed":"400","overHeal":"50","enchps":"4"}}}"#;
//     let options = ParseOptions { merge_pets_into_owner: true, local_player_name: Some("Future Fade") };
//     let OverlayMessage::CombatData(merged) = OverlayMessage::parse(json, options).unwrap() else {
//         panic!("expected combat data")
//     };
//     let you = merged.combatant("YOU").unwrap();
//     assert_eq!(you.healing().effective(), 650.0); // 1000 - 150 overheal - 200 shield
//     assert_eq!(you.pet_effective_healed, 350.0); // Eos: 400 - 50
//     assert_eq!(you.heal_per_second, 10.0);
// }
