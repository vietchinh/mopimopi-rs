//! Unit tests for `act_data::combat_data_message`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

pub const SAMPLE_FIGHT_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/previewLog.json"));

#[test]
fn parses_the_sample_fight() {
    let message: CombatDataMessage = serde_json::from_str(SAMPLE_FIGHT_JSON).unwrap();
    assert_eq!(message.encounter.title, "Striking Dummy");
    assert_eq!(message.encounter.duration_seconds, 35.0);
    assert_eq!(message.combatants.len(), 4);
    // arrival order is preserved
    assert_eq!(message.combatants[0].name, "Eos (YOU)");
    let you = message.combatants.iter().find(|c| c.name == "YOU").unwrap();
    assert_eq!(you.job_text, "Sch");
    assert_eq!(you.damage, 77193.0);
    assert_eq!(you.strongest_hit_text, "Broil-8,765");
    assert_eq!(you.strongest_hit_amount, 8765.0);
}

#[test]
fn active_flag_accepts_text() {
    let message: CombatDataMessage = serde_json::from_str(r#"{"Encounter":{},"isActive":"true"}"#).unwrap();
    assert!(message.is_encounter_active);
    assert!(message.combatants.is_empty());
}

#[test]
fn missing_encounter_is_an_error() {
    assert!(serde_json::from_str::<CombatDataMessage>(r#"{"Combatant":{}}"#).is_err());
}
