//! A player's strongest hit or heal: "Broil-8,765" -> action "Broil", amount 8765.

use crate::models::act_data::describes_a_name_not_a_number;

const NO_DATA_LABEL: &str = "No Data";

#[derive(Clone, Debug, PartialEq)]
pub struct StrongestAction {
    pub action_name: String,
    pub amount: f64,
}

impl StrongestAction {
    pub fn no_data() -> StrongestAction {
        StrongestAction { action_name: NO_DATA_LABEL.to_string(), amount: 0.0 }
    }

    /// `descriptive_text` is ACT's "Action-1,234" string, `amount` its numeric companion field.
    /// Plain numbers or empty text mean the player had no such action.
    pub fn from_act_fields(descriptive_text: &str, amount: f64) -> StrongestAction {
        if !describes_a_name_not_a_number(descriptive_text) {
            return StrongestAction::no_data();
        }
        let action_name = descriptive_text.split('-').next().unwrap_or("").to_string();
        StrongestAction { action_name, amount }
    }

    /// True when this action is stronger than `other` (used when folding pets into owners).
    pub fn beats(&self, other: &StrongestAction) -> bool {
        self.amount > other.amount
    }
}

impl Default for StrongestAction {
    fn default() -> Self {
        StrongestAction::no_data()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/strongest_action.rs"]
mod tests;
