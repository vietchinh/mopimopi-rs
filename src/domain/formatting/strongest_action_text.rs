//! The MaxHit / MaxHeal cell: action name (optionally abbreviated) and amount.

use super::number_format::NumberFormat;
use super::text_fragment::TextFragment;
use crate::domain::combat::StrongestAction;
use crate::domain::settings::Settings;

/// Where the action name goes relative to the amount ("MaxHit" settings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionLayout {
    NameThenAmount,
    AmountThenName,
    NameOnly,
    AmountOnly,
}

impl ActionLayout {
    fn from_option_number(number: i32) -> ActionLayout {
        match number {
            1 => ActionLayout::NameThenAmount,
            2 => ActionLayout::AmountThenName,
            3 => ActionLayout::NameOnly,
            _ => ActionLayout::AmountOnly,
        }
    }
}

/// Fragments of the cell for `action`.
pub fn strongest_action_fragments(action: &StrongestAction, settings: &Settings, number_format: &NumberFormat) -> Vec<TextFragment> {
    let action_name = shown_action_name(&action.action_name, settings);
    let amount = number_format.strongest_action_amount_fragments(action.amount);
    let separator = settings.option_text("mhh_unit");
    match ActionLayout::from_option_number(settings.option_number("mhh") as i32) {
        ActionLayout::NameThenAmount => {
            let mut fragments = vec![TextFragment::Dimmed(format!("{action_name} {separator} "))];
            fragments.extend(amount);
            fragments
        }
        ActionLayout::AmountThenName => {
            let mut fragments = amount;
            fragments.push(TextFragment::Dimmed(format!(" {separator} {action_name}")));
            fragments
        }
        ActionLayout::NameOnly => vec![TextFragment::Plain(action_name)],
        ActionLayout::AmountOnly => amount,
    }
}

/// The user's abbreviation for the action, when abbreviations are on and one is saved.
fn shown_action_name(action_name: &str, settings: &Settings) -> String {
    if !settings.option_enabled("abb") {
        return action_name.to_string();
    }
    settings
        .action_abbreviations()
        .into_iter()
        .find(|(long_name, _)| long_name == action_name)
        .map(|(_, short_name)| short_name)
        .unwrap_or_else(|| action_name.to_string())
}
