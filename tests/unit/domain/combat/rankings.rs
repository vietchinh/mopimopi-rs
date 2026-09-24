//! Unit tests for `combat::rankings`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

fn sample_message() -> CombatDataMessage {
    serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/previewLog.json"))).unwrap()
}

#[test]
fn pets_merge_into_their_owners() {
    let rankings = build_rankings(&sample_message(), true, "");
    let by_damage = &rankings.by_damage;
    // Eos and Rook Autoturret are pets and hidden; YOU and Player Mch remain.
    assert_eq!(by_damage.party_size, 2);
    let machinist = by_damage.player_named("Player Mch").unwrap();
    assert_eq!(machinist.merged_stats.damage, 128399.0 + 29046.0);
    assert_eq!(by_damage.players[0].name, "Player Mch");
    // Eos' healing is folded into YOU (the pets' owner name matches no real player).
    let local_healing = rankings.by_healing.local_player().unwrap();
    assert_eq!(local_healing.merged_stats.healed, 12852.0 + 47644.0);
    assert_eq!(rankings.by_healing.players[0].name, "YOU");
    assert_eq!(by_damage.local_player().unwrap().merged_stats.damage, 77193.0);
}

#[test]
fn pets_stay_separate_when_merging_is_off() {
    let rankings = build_rankings(&sample_message(), false, "");
    assert_eq!(rankings.by_damage.party_size, 4);
    assert_eq!(rankings.by_damage.player_named("Player Mch").unwrap().merged_stats.damage, 128399.0);
    assert_eq!(rankings.by_damage.players.len(), 4);
}

#[test]
fn classification_and_strongest_hit() {
    let rankings = build_rankings(&sample_message(), false, "");
    let local = rankings.by_damage.local_player().unwrap();
    assert_eq!(local.class_code, "SCH");
    assert_eq!(local.role, super::super::PlayerRole::Healer);
    assert_eq!(local.own_strongest_hit.action_name, "Broil");
    assert_eq!(local.own_strongest_hit.amount, 8765.0);
    let pet = rankings.by_damage.player_named("Eos (YOU)").unwrap();
    assert_eq!(pet.job_code, "AVA");
    assert_eq!(pet.own_strongest_hit.action_name, "No Data");
    assert_eq!(pet.pet_owner_name, "YOU");
}

#[test]
fn derived_values_and_encounter_key() {
    let rankings = build_rankings(&sample_message(), false, "");
    let local = rankings.by_damage.local_player().unwrap();
    assert!((local.rates.encounter_damage_per_second - 2205.51).abs() < 0.01);
    assert!((local.rates.damage_percent - 32.9).abs() < 0.1);
    assert_eq!(rankings.by_damage.encounter_key, "Striking Dummy23463860496");
}
