//! Numbers from the Zurvan log; ACT's own rounded values in the comments.

use super::*;

fn approx(actual: Option<f64>, expected: f64) {
    let actual = actual.expect("expected a percentage");
    assert!((actual - expected).abs() < 0.01, "{actual} != {expected}");
}

#[test]
fn scholar() {
    let hamtaro = Healing { healed: 276_963.0, over_heal: 72_660.0, damage_shield: 58_310.0 };
    assert_eq!(hamtaro.effective(), 145_993.0);
    approx(hamtaro.overheal_percent(), 26.23); // ACT: "26%"
    approx(hamtaro.shield_percent(), 21.05);
}

#[test]
fn paladin_whose_healing_is_all_shield() {
    let llouss = Healing { healed: 18_880.0, over_heal: 0.0, damage_shield: 18_880.0 };
    assert_eq!(llouss.effective(), 0.0);
    approx(llouss.shield_percent(), 100.0);
    approx(llouss.overheal_percent(), 0.0); // ACT: "0%"
}

#[test]
fn no_healing_has_no_percentages() {
    let monk = Healing::default();
    assert_eq!(monk.effective(), 0.0);
    assert_eq!(monk.overheal_percent(), None);
    assert_eq!(monk.shield_percent(), None);
}
