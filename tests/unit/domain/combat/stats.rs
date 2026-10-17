use super::*;

fn stats(name: &str, job: &str) -> Stats {
    Stats { name: name.into(), job: job.to_string().into(), ..Stats::default() }
}

#[test]
fn absorbing_a_pet_adds_its_numbers_keeps_the_stronger_hit_and_remembers_the_pets_share() {
    let mut owner = Stats { damage: Amount::new(100.0), strongest_hit_amount: Amount::new(50.0), strongest_hit_name: "Mine".into(), ..stats("YOU", "Smn") };
    let pet = Stats { damage: Amount::new(40.0), strongest_hit_amount: Amount::new(80.0), strongest_hit_name: "Theirs".into(), ..stats("Carbuncle (YOU)", "") };
    owner.absorb_pet(&pet);
    assert_eq!((owner.damage, owner.pet_damage), (Amount::new(140.0), Amount::new(40.0)));
    assert_eq!((owner.strongest_hit_amount, owner.strongest_hit_name.as_str()), (Amount::new(80.0), "Theirs"));
}

#[test]
fn personal_dps_uses_the_members_own_time_and_treats_zero_as_one_second() {
    let timed = Stats { damage: Amount::new(1000.0), duration_seconds: 10.0, ..stats("A", "Drk") };
    assert_eq!(timed.personal_damage_per_second().value(), 100.0);
    let untimed = Stats { damage: Amount::new(1234.0), ..stats("A", "Drk") };
    assert_eq!(untimed.personal_damage_per_second().value(), 1234.0);
}
