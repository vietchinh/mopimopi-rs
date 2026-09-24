//! Unit tests for `combat::strongest_action`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn splits_name_from_amount() {
    let action = StrongestAction::from_act_fields("Broil-8,765", 8765.0);
    assert_eq!(action.action_name, "Broil");
    assert_eq!(action.amount, 8765.0);
}

#[test]
fn empty_or_numeric_text_means_no_data() {
    assert_eq!(StrongestAction::from_act_fields("", 0.0), StrongestAction::no_data());
    assert_eq!(StrongestAction::from_act_fields("123", 5.0), StrongestAction::no_data());
}
