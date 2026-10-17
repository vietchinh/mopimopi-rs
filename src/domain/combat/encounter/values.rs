//! Value objects of the Encounter: immutable, compared by value, valid by construction.
//!
//! Each one carries the original overlay's rounding in its constructor, so no caller has to remember to apply it
//! (DDD: value objects; talk rule 5: preconditions in types).

use crate::domain::javascript_number::p_float;
use std::fmt;

/// A quantity of damage or healing. NaN cannot exist.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Amount(f64);

impl Amount {
    pub fn new(value: f64) -> Self {
        Self(if value.is_nan() { 0.0 } else { value })
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl std::ops::Add for Amount {
    type Output = Amount;
    fn add(self, other: Amount) -> Amount {
        Amount(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Amount {
    fn add_assign(&mut self, other: Amount) {
        *self = *self + other;
    }
}

/// A rate the program derives (amount over seconds), rounded to two decimals like the original's `pFloat`. A per-second
/// figure that ACT itself reports is a `Reported`, which the original never rounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Rate(f64);

impl Rate {
    /// Rounds to two decimals; NaN becomes 0, infinity stays.
    pub fn new(value: f64) -> Self {
        Self(p_float(value))
    }

    /// `amount` over `seconds`. Zero seconds give infinity (or 0 for no amount), as in JavaScript.
    pub fn per_second(amount: Amount, seconds: f64) -> Self {
        Self::new(amount.value() / seconds)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl std::ops::Add for Rate {
    type Output = Rate;
    fn add(self, other: Rate) -> Rate {
        Rate(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Rate {
    fn add_assign(&mut self, other: Rate) {
        *self = *self + other;
    }
}

/// A ratio or per-second figure exactly as ACT reported it. The original shows these as they came, without its `pFloat`
/// rounding, so neither do we; only NaN cannot exist.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Reported(f64);

impl Reported {
    pub fn new(value: f64) -> Self {
        Self(if value.is_nan() { 0.0 } else { value })
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl std::ops::Add for Reported {
    type Output = Reported;
    fn add(self, other: Reported) -> Reported {
        Reported(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Reported {
    fn add_assign(&mut self, other: Reported) {
        *self = *self + other;
    }
}

/// A share of a whole that the program derives, in percent, rounded to two decimals like `pFloat`. 0 of 0 is 0; more than 0
/// of 0 is infinity, as in JavaScript.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Percent(f64);

impl Percent {
    pub fn new(value: f64) -> Self {
        Self(p_float(value))
    }

    /// The share `part` makes of `whole`; 0 when there is no whole (ACT would show NaN there).
    pub fn share(part: Amount, whole: Amount) -> Self {
        if whole.value() == 0.0 {
            Self::default()
        } else {
            Self::of(part, whole)
        }
    }

    pub fn of(part: Amount, whole: Amount) -> Self {
        Self::new(part.value() / whole.value() * 100.0)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

/// The job text as the row carries it. Kept as text; what it means for the kind of participant is decided at the ACT edge.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Job(String);

impl Job {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Job {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for Job {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for Job {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl From<String> for Job {
    fn from(text: String) -> Self {
        Self(text)
    }
}

impl fmt::Display for Job {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/domain/combat/encounter_values.rs"]
mod tests;
