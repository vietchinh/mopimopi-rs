use super::*;

fn member(name: &str, job: &str) -> Member {
    Member { name: name.into(), job_text: job.into(), ..Member::default() }
}

#[test]
fn base_classes_take_their_advanced_jobs_colour_and_the_limit_break_is_lmb() {
    assert_eq!(member("A", "Gla").class_code(), "PLD");
    assert_eq!(member("A", "Drk").class_code(), "DRK");
    assert_eq!(member("Limit Break", "Limit Break").class_code(), "LMB");
    assert_eq!(member("Chocobo (YOU)", "0").class_code(), "CBO");
}

#[test]
fn a_scholars_pet_counts_as_a_healer_and_a_dark_knights_as_a_tank() {
    assert!(member("Eos (YOU)", "").is_healer());
    assert!(member("Esteem (YOU)", "").is_tank());
    assert!(!member("Carbuncle (YOU)", "").is_healer());
}

#[test]
fn absorbing_a_pet_adds_its_numbers_keeps_the_stronger_hit_and_remembers_the_pets_share() {
    let mut owner = Member { damage: 100.0, strongest_hit_amount: 50.0, strongest_hit_name: "Mine".into(), ..member("YOU", "Smn") };
    let pet = Member { damage: 40.0, strongest_hit_amount: 80.0, strongest_hit_name: "Theirs".into(), ..member("Carbuncle (YOU)", "") };
    owner.absorb_pet(&pet);
    assert_eq!((owner.damage, owner.pet_damage), (140.0, 40.0));
    assert_eq!((owner.strongest_hit_amount, owner.strongest_hit_name.as_str()), (80.0, "Theirs"));
}

#[test]
fn personal_dps_uses_the_members_own_time_and_treats_zero_as_one_second() {
    let timed = Member { damage: 1000.0, duration_seconds: 10.0, ..member("A", "Drk") };
    assert_eq!(timed.personal_damage_per_second().value(), 100.0);
    let untimed = Member { damage: 1234.0, ..member("A", "Drk") };
    assert_eq!(untimed.personal_damage_per_second().value(), 1234.0);
}

#[test]
fn a_jobless_unowned_row_is_an_npc_but_the_gauge_is_not() {
    assert!(member("Aw'aern", "").is_npc());
    assert!(!member("Limit Break", "").is_npc(), "named like the gauge, no job");
    assert!(!member("Limit Break", "Limit Break").is_npc());
    assert!(!member("Alice", "Drk").is_npc());
    assert!(!member("Eos (YOU)", "").is_npc(), "a pet is not an NPC");
}
