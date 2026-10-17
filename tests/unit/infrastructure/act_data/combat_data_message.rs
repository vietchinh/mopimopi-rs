//! Unit tests for `infrastructure::act::data::combat_data_message`. Compiled as a child module of
//! that file (so they can use its private items) but kept here, outside `src/`.

use super::*;
use crate::domain::combat::Participant;
use crate::domain::combat::PetView;

fn parse(json: &str) -> Encounter {
    encounter_from_json(json).expect("valid combat data")
}

const FOLDED: PetView<'static> = PetView::FoldedIntoOwners { local_player_name: None };

fn names(rows: &[Participant]) -> Vec<&str> {
    rows.iter().map(|row| row.name.as_str()).collect()
}

#[test]
fn keys_are_dropped_and_arrival_order_is_kept() {
    let message = parse(r#"{"Encounter":{},"Combatant":{"B":{"name":"B","damage":"1"},"A":{"name":"A","damage":"2"}}}"#);
    assert_eq!(names(message.participants()), ["B", "A"], "the object's own key order is kept; keys are not used to sort");
}

#[test]
fn the_local_player_is_the_row_called_you() {
    let message = parse(r#"{"Encounter":{},"Combatant":{"YOU":{"name":"YOU","damage":"5"}}}"#);
    assert_eq!(message.you().unwrap().damage, 5.0);
    assert!(parse(r#"{"Encounter":{},"Combatant":{"Nobody":{"name":"Nobody"}}}"#).you().is_none());
}

#[test]
fn folding_with_no_pets_present_changes_nothing_and_rows_are_sorted_by_damage() {
    // Deliberately NOT in damage order.
    let message = parse(r#"{"Encounter":{},"Combatant":{
        "A":{"name":"A","damage":"10"},
        "B":{"name":"B","damage":"90"}
    }}"#);
    assert_eq!(names(message.participants()), ["A", "B"], "the Encounter keeps ACT's order");
    assert_eq!(names(&message.rows(FOLDED)), ["B", "A"], "the original always sorts by damage");
    assert_eq!(names(&message.rows(PetView::Separate)), ["B", "A"]);
}

#[test]
fn folding_a_pet_removes_its_row_and_adds_its_numbers_to_the_owner() {
    let message = parse(r#"{"Encounter":{},"Combatant":{
        "YOU":{"name":"YOU","Job":"Sch","damage":"100","healed":"50"},
        "Eos (YOU)":{"name":"Eos (YOU)","damage":"20","healed":"5"}
    }}"#);
    assert_eq!(message.participants().len(), 2, "the Encounter itself keeps the pet");
    let rows = message.rows(FOLDED);
    assert_eq!(rows.len(), 1, "the pet's row is gone from the view");
    let owner = &rows[0];
    assert_eq!((owner.damage, owner.healed, owner.pet_damage), (120.0, 55.0, 20.0));
}

#[test]
fn folding_re_sorts_by_damage_only_when_a_merge_actually_disturbs_the_order() {
    // Ahead of the summoner even after merging: order must not change.
    let untouched = parse(r#"{"Encounter":{},"Combatant":{
        "Leader":{"name":"Leader","damage":"500"},
        "YOU":{"name":"YOU","Job":"Smn","damage":"50"},
        "Carbuncle (YOU)":{"name":"Carbuncle (YOU)","damage":"20"}
    }}"#);
    let rows = untouched.rows(FOLDED);
    assert_eq!(names(&rows), ["Leader", "YOU"], "70 < 500: still behind Leader, order is unchanged");

    // The summoner's own damage plus the pet's now beats the player ACT had ranked above them.
    let overtakes = parse(r#"{"Encounter":{},"Combatant":{
        "Leader":{"name":"Leader","damage":"100"},
        "YOU":{"name":"YOU","Job":"Smn","damage":"50"},
        "Carbuncle (YOU)":{"name":"Carbuncle (YOU)","damage":"90"}
    }}"#);
    let rows = overtakes.rows(FOLDED);
    assert_eq!(names(&rows), ["YOU", "Leader"], "140 > 100: the merged owner now leads, so it moved to the front");
}

#[test]
fn the_sort_key_is_damage() {
    let message = parse(r#"{"Encounter":{"DURATION":"20"},"Combatant":{
        "A":{"name":"A","damage":"100"},
        "B":{"name":"B","Job":"Smn","damage":"50"},
        "Carbuncle (B)":{"name":"Carbuncle (B)","damage":"90"}
    }}"#);
    // B's folded damage (50+90) is ahead of A's 100.
    assert_eq!(names(&message.rows(FOLDED)), ["B", "A"]);
}

#[test]
fn a_zero_damage_zero_duration_nan_rate_still_falls_back_to_raw_damage() {
    // duration = 0 and damage = 0 together make `damage / duration` NaN, not infinity, for that one
    // row; NaN must not make the comparison silently give up and call it a tie against a real value.
    let message = parse(r#"{"Encounter":{},"Combatant":{
        "Empty":{"name":"Empty","damage":"0"},
        "YOU":{"name":"YOU","Job":"Smn","damage":"50"},
        "Carbuncle (YOU)":{"name":"Carbuncle (YOU)","damage":"90"}
    }}"#);
    let rows = message.rows(FOLDED);
    assert_eq!(names(&rows), ["YOU", "Empty"], "140 damage clearly outranks 0, NaN or not");
}

#[test]
fn folding_uses_the_local_player_name_to_find_the_players_own_pets() {
    // The pet's row says the real character name, not "YOU"; the local player name bridges that.
    let message = parse(r#"{"Encounter":{},"Combatant":{
        "YOU":{"name":"YOU","Job":"Sch","damage":"10"},
        "Eos (Future Fade)":{"name":"Eos (Future Fade)","damage":"5"}
    }}"#);
    let rows = message.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") });
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].damage, 15.0);
}
