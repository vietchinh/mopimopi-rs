//! Unit tests for `combat::job_classification`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn scholar_is_a_healer() {
    let result = classify("YOU", "Sch");
    assert_eq!(result.class_code, "SCH");
    assert_eq!(result.role, PlayerRole::Healer);
    assert!(!result.is_pet);
}

#[test]
fn base_class_uses_advanced_job_icon_and_role() {
    let result = classify("Someone", "Gla");
    assert_eq!(result.class_code, "PLD");
    assert_eq!(result.role, PlayerRole::Tank);
}

#[test]
fn known_pet_is_recognised_by_name() {
    let result = classify("Eos (YOU)", "");
    assert_eq!(result.job_code, PET_JOB_CODE);
    assert_eq!(result.class_code, "SCH");
    assert_eq!(result.role, PlayerRole::Healer);
    assert_eq!(result.pet_owner_name, "YOU");
}

#[test]
fn unknown_owned_combatant_becomes_cbo() {
    let result = classify("Wild Thing (YOU)", "");
    assert_eq!(result.job_code, COMBATANT_JOB_CODE);
    assert_eq!(result.role, PlayerRole::OwnedCombatant);
}

#[test]
fn nameless_npc_without_owner_is_limit_break() {
    let result = classify("Limit Break", "");
    assert_eq!(result.class_code, LIMIT_BREAK_JOB_CODE);
}
