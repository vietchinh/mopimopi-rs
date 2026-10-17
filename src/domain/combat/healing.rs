//! What a participant's healing numbers mean.
//!
//! ACT's `healed` is everything: HP restored, overheal, and shields (`damageShield`). A
//! Paladin whose only "healing" is Divine Veil has `healed` equal to `damageShield`.

use super::encounter::{Amount, Percent};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Healing {
    pub healed: Amount,
    pub over_heal: Amount,
    pub damage_shield: Amount,
}

impl Healing {
    /// HP actually restored: everything minus overheal minus shields (mopimopi's "`EffHealed`").
    pub fn effective(self) -> Amount {
        Amount::new(self.healed.value() - self.over_heal.value() - self.damage_shield.value())
    }

    /// ACT's `OverHealPct`, rounded like the original; 0 when nothing was healed.
    pub fn overheal_share(self) -> Percent {
        Percent::share(self.over_heal, self.healed)
    }

    /// How much of `healed` was shields (mopimopi draws this on the HPS bar). Not read by any
    /// display code yet (nothing shows a shield percentage column), but it is real, correct
    /// behaviour with its own tests (`tests/unit/domain/combat/healing.rs`), not leftover cruft.
    #[allow(dead_code)]
    pub fn shield_share(self) -> Percent {
        Percent::share(self.damage_shield, self.healed)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/healing.rs"]
mod tests;
