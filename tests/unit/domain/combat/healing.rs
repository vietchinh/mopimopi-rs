//! Numbers from the Zurvan log; ACT's own rounded values in the comments.

use super::*;
use crate::domain::combat::encounter::Percent;

fn approx(actual: Percent, expected: f64) {
    let actual = actual.value();
    assert!((actual - expected).abs() < 0.01, "{actual} != {expected}");
}

#[test]
fn scholar() {
    let hamtaro = Healing { healed: 276_963.0, over_heal: 72_660.0, damage_shield: 58_310.0 };
    assert_eq!(hamtaro.effective(), 145_993.0);
    approx(hamtaro.overheal_share(), 26.23); // ACT: "26%"
    approx(hamtaro.shield_share(), 21.05);
}

#[test]
fn paladin_whose_healing_is_all_shield() {
    let llouss = Healing { healed: 18_880.0, over_heal: 0.0, damage_shield: 18_880.0 };
    assert_eq!(llouss.effective(), 0.0);
    approx(llouss.shield_share(), 100.0);
    approx(llouss.overheal_share(), 0.0); // ACT: "0%"
}

#[test]
fn no_healing_has_zero_percentages() {
    let monk = Healing::default();
    assert_eq!(monk.effective(), 0.0);
    assert_eq!(monk.overheal_share().value(), 0.0);
    assert_eq!(monk.shield_share().value(), 0.0);
}
