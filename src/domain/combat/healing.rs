//! What a member's healing numbers mean.
//!
//! ACT's `healed` is everything: HP restored, overheal, and shields (`damageShield`). A
//! Paladin whose only "healing" is Divine Veil has `healed` equal to `damageShield`.

use super::encounter::{Amount, Percent};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Healing {
    pub healed: f64,
    pub over_heal: f64,
    pub damage_shield: f64,
}

impl Healing {
    /// HP actually restored: everything minus overheal minus shields (mopimopi's "`EffHealed`").
    pub fn effective(self) -> f64 {
        self.healed - self.over_heal - self.damage_shield
    }

    /// ACT's `OverHealPct`, rounded like the original; 0 when nothing was healed.
    pub fn overheal_share(self) -> Percent {
        Percent::share(Amount::new(self.over_heal), Amount::new(self.healed))
    }

    /// How much of `healed` was shields (mopimopi draws this on the HPS bar). Not read by any
    /// display code yet (nothing shows a shield percentage column), but it is real, correct
    /// behaviour with its own tests (`tests/unit/domain/combat/healing.rs`), not leftover cruft.
    #[allow(dead_code)]
    pub fn shield_share(self) -> Percent {
        Percent::share(Amount::new(self.damage_shield), Amount::new(self.healed))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/healing.rs"]
mod tests;
