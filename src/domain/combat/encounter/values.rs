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

/// An amount per second, rounded to two decimals like the original's `pFloat`.
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

/// A share of a whole, in percent, rounded to two decimals like `pFloat`. 0 of 0 is 0; more than 0 of 0 is infinity, as in JavaScript.
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

/// The job text ACT reports ("Drk", "Limit Break", "" for NPCs, "0" for chocobos). Kept as text; what it means is
/// decided once, when the Encounter is assembled.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Job(String);

impl Job {
    pub fn new(text: &str) -> Self {
        Self(text.to_string())
    }

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
