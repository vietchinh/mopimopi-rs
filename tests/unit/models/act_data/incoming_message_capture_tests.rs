//! Unit tests for `act_data::incoming_message`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

/// A real MiniParse capture: Beastmaster with a companion, a `type` repeated inside `msg`,
/// `NaN` and `--` placeholders, and the generic "Encounter" title.
const BEASTMASTER_CAPTURE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/captures/mini_parse_beastmaster.json"));

#[test]
fn parses_a_real_mini_parse_capture() {
    let Some(ActEvent::CombatData(message)) = parse_incoming_message(BEASTMASTER_CAPTURE).unwrap() else {
        panic!("expected combat data");
    };
    assert!(message.is_encounter_active);
    assert_eq!(message.encounter.zone_name, "Shirogane");
    assert_eq!(message.combatants.len(), 2);
    let you = message.combatants.iter().find(|c| c.name == "YOU").unwrap();
    assert_eq!(you.job_text, "Bst");
    assert_eq!(you.damage, 1714.0);
    assert_eq!(you.strongest_hit_text, "attack-53");
}

#[test]
fn companion_without_job_is_an_owned_combatant() {
    use crate::domain::combat::{build_rankings, PlayerRole};
    let Some(ActEvent::CombatData(message)) = parse_incoming_message(BEASTMASTER_CAPTURE).unwrap() else { panic!() };
    let rankings = build_rankings(&message, true, "");
    let companion = rankings.by_damage.player_named("Cu Sith (YOU)").unwrap();
    assert_eq!(companion.role, PlayerRole::OwnedCombatant);
    assert_eq!(companion.pet_owner_name, "YOU");
    assert_eq!(rankings.by_damage.local_player().unwrap().class_code, "BST");
}
