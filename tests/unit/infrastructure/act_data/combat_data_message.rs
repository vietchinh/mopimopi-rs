//! Unit tests for `infrastructure::act::data::combat_data_message`. Compiled as a child module of
//! that file (so they can use its private items) but kept here, outside `src/`.

use super::*;

fn parse(json: &str) -> CombatDataMessage {
    serde_json::from_str(json).expect("valid combat data")
}

#[test]
fn keys_are_dropped_and_arrival_order_is_kept() {
    let message = parse(r#"{"Encounter":{},"Combatant":{"B":{"name":"B","damage":"1"},"A":{"name":"A","damage":"2"}}}"#);
    let names: Vec<&str> = message.combatants.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["B", "A"], "the object's own key order is kept; keys are not used to sort");
}

#[test]
fn local_player_and_combatant_lookup() {
    let message = parse(r#"{"Encounter":{},"Combatant":{"YOU":{"name":"YOU","damage":"5"}}}"#);
    assert_eq!(message.local_player().unwrap().name, "YOU");
    assert_eq!(message.combatant("YOU").unwrap().damage, 5.0);
    assert!(message.combatant("Nobody").is_none());
}

#[test]
fn merging_with_no_pets_present_changes_nothing_and_does_not_resort() {
    // Deliberately NOT in damage order: with no pets to merge, nothing should touch this.
    let mut message = parse(r#"{"Encounter":{},"Combatant":{
        "A":{"name":"A","damage":"10"},
        "B":{"name":"B","damage":"90"}
    }}"#);
    message.merge_pets_into_owners(None);
    let names: Vec<&str> = message.combatants.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["A", "B"], "no pets were merged, so ACT's (here: the test's) order is left alone");
}

#[test]
fn merging_a_pet_removes_its_row_and_adds_its_numbers_to_the_owner() {
    let mut message = parse(r#"{"Encounter":{},"Combatant":{
        "YOU":{"name":"YOU","Job":"Sch","damage":"100","healed":"50"},
        "Eos (YOU)":{"name":"Eos (YOU)","damage":"20","healed":"5"}
    }}"#);
    message.merge_pets_into_owners(None);
    assert_eq!(message.combatants.len(), 1, "the pet's row is gone");
    let owner = message.local_player().unwrap();
    assert_eq!((owner.damage, owner.healed, owner.pet_damage), (120.0, 55.0, 20.0));
}

#[test]
fn merging_re_sorts_by_damage_only_when_a_merge_actually_disturbs_the_order() {
    // Ahead of the summoner even after merging: order must not change.
    let mut untouched = parse(r#"{"Encounter":{},"Combatant":{
        "Leader":{"name":"Leader","damage":"500"},
        "YOU":{"name":"YOU","Job":"Smn","damage":"50"},
        "Carbuncle (YOU)":{"name":"Carbuncle (YOU)","damage":"20"}
    }}"#);
    untouched.merge_pets_into_owners(None);
    let names: Vec<&str> = untouched.combatants.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Leader", "YOU"], "70 < 500: still behind Leader, order is unchanged");

    // The summoner's own damage plus the pet's now beats the player ACT had ranked above them.
    let mut overtakes = parse(r#"{"Encounter":{},"Combatant":{
        "Leader":{"name":"Leader","damage":"100"},
        "YOU":{"name":"YOU","Job":"Smn","damage":"50"},
        "Carbuncle (YOU)":{"name":"Carbuncle (YOU)","damage":"90"}
    }}"#);
    overtakes.merge_pets_into_owners(None);
    let names: Vec<&str> = overtakes.combatants.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["YOU", "Leader"], "140 > 100: the merged owner now leads, so it moved to the front");
}

#[test]
fn the_resort_key_is_encdps_not_raw_damage() {
    // Everyone shares the same encounter duration, so this is order-equivalent to a raw damage sort
    // in the common case, but it exercises the actual encdps division rather than only the
    // duration = 0 fallback the other tests happen to hit (since `Encounter:{}` there has no DURATION).
    let mut message = parse(r#"{"Encounter":{"DURATION":"20"},"Combatant":{
        "A":{"name":"A","damage":"100"},
        "B":{"name":"B","Job":"Smn","damage":"50"},
        "Carbuncle (B)":{"name":"Carbuncle (B)","damage":"90"}
    }}"#);
    message.merge_pets_into_owners(None);
    let names: Vec<&str> = message.combatants.iter().map(|c| c.name.as_str()).collect();
    // B's merged encdps is (50+90)/20 = 7.0, ahead of A's 100/20 = 5.0.
    assert_eq!(names, ["B", "A"]);
}

#[test]
fn a_zero_damage_zero_duration_nan_rate_still_falls_back_to_raw_damage() {
    // duration = 0 and damage = 0 together make `damage / duration` NaN, not infinity, for that one
    // row; NaN must not make the comparison silently give up and call it a tie against a real value.
    let mut message = parse(r#"{"Encounter":{},"Combatant":{
        "Empty":{"name":"Empty","damage":"0"},
        "YOU":{"name":"YOU","Job":"Smn","damage":"50"},
        "Carbuncle (YOU)":{"name":"Carbuncle (YOU)","damage":"90"}
    }}"#);
    message.merge_pets_into_owners(None);
    let names: Vec<&str> = message.combatants.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["YOU", "Empty"], "140 damage clearly outranks 0, NaN or not");
}

#[test]
fn merging_uses_the_local_player_name_to_find_the_players_own_pets() {
    // The pet's row says the real character name, not "YOU"; the local player name bridges that.
    let mut message = parse(r#"{"Encounter":{},"Combatant":{
        "YOU":{"name":"YOU","Job":"Sch","damage":"10"},
        "Eos (Future Fade)":{"name":"Eos (Future Fade)","damage":"5"}
    }}"#);
    message.merge_pets_into_owners(Some("Future Fade"));
    assert_eq!(message.combatants.len(), 1);
    assert_eq!(message.local_player().unwrap().damage, 15.0);
}
