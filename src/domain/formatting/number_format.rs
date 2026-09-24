//! Number formatting driven by the "Number" settings page.

use super::text_fragment::TextFragment;
use crate::common::javascript_compat::to_fixed;
use crate::domain::settings::Settings;

const THOUSAND: f64 = 1_000.0;
const MILLION: f64 = 1_000_000.0;
/// Values from this size on are shortened to k / M when the unit option is on.
const MINIMUM_FOR_THOUSANDS_UNIT: f64 = 10_000.0;

/// How numbers are written, read once from the settings.
#[derive(Clone, Debug, PartialEq)]
pub struct NumberFormat {
    /// Separator between digit groups; `None` when grouping is disabled.
    pub thousands_separator: Option<String>,
    pub decimal_separator: String,
    /// Decimals for DPS / HPS values.
    pub rate_decimals: usize,
    /// Decimals for percentages.
    pub percent_decimals: usize,
    /// Decimals for damage / healing amounts (only used when shortened to k / M).
    pub amount_decimals: usize,
    /// Show large DPS and damage values as "12.3k" / "1.2M".
    pub shorten_large_values: bool,
    /// Show large MaxHit / MaxHeal values as "12.3k" / "1.2M".
    pub shorten_strongest_action_values: bool,
}

impl NumberFormat {
    pub fn from_settings(settings: &Settings) -> NumberFormat {
        let digits_after_decimal = settings.option_number("ns");
        let decimals_for = |kind_option: &str| (digits_after_decimal * settings.option_number(kind_option)).max(0.0) as usize;
        let grouping = settings.option_text("gs");
        let decimal_mark = settings.option_text("ds");
        NumberFormat {
            thousands_separator: match grouping.as_str() {
                "" | "0" => None,
                "_" => Some(" ".to_string()),
                other => Some(other.to_string()),
            },
            decimal_separator: match decimal_mark.as_str() {
                "" => ".".to_string(),
                "_" => " ".to_string(),
                other => other.to_string(),
            },
            rate_decimals: decimals_for("dpsType"),
            percent_decimals: decimals_for("perType"),
            amount_decimals: decimals_for("dmgType"),
            shorten_large_values: settings.option_enabled("unit"),
            shorten_strongest_action_values: settings.option_enabled("max_unit"),
        }
    }

    /// `value / divisor` with grouping and decimal marks; non-finite values print as "0".
    pub fn format_number(&self, value: f64, divisor: f64, decimals: usize) -> String {
        if !value.is_finite() {
            return "0".into();
        }
        let fixed = to_fixed(value / divisor, decimals);
        let (integer_part, fraction) = match fixed.split_once('.') {
            Some((integer, fraction)) => (integer.to_string(), Some(fraction.to_string())),
            None => (fixed, None),
        };
        let grouped = match &self.thousands_separator {
            Some(separator) => group_thousands(&integer_part, separator),
            None => integer_part,
        };
        match fraction {
            Some(fraction) => format!("{grouped}{}{fraction}", self.decimal_separator),
            None => grouped,
        }
    }

    /// The number followed by a dimmed unit when `divisor` is 1,000 ("k") or 1,000,000 ("M").
    pub fn number_with_unit(&self, value: f64, divisor: f64, decimals: usize) -> Vec<TextFragment> {
        let mut fragments = vec![TextFragment::Plain(self.format_number(value, divisor, decimals))];
        if value.is_finite() {
            if divisor == THOUSAND {
                fragments.push(TextFragment::Dimmed("k".into()));
            } else if divisor == MILLION {
                fragments.push(TextFragment::Dimmed("M".into()));
            }
        }
        fragments
    }

    /// A DPS / HPS value: "∞" for infinity, shortened to k when large and enabled.
    pub fn rate_fragments(&self, value: f64) -> Vec<TextFragment> {
        if value == f64::INFINITY {
            return vec![TextFragment::Plain("∞".into())];
        }
        let divisor = if self.shorten_large_values && value >= MINIMUM_FOR_THOUSANDS_UNIT { THOUSAND } else { 1.0 };
        self.number_with_unit(value, divisor, self.rate_decimals)
    }

    /// A damage / healing amount: k / M when large and enabled, otherwise a whole number.
    pub fn amount_fragments(&self, value: f64) -> Vec<TextFragment> {
        if self.shorten_large_values && value >= MILLION {
            self.number_with_unit(value, MILLION, self.amount_decimals)
        } else if self.shorten_large_values && value >= MINIMUM_FOR_THOUSANDS_UNIT {
            self.number_with_unit(value, THOUSAND, self.amount_decimals)
        } else {
            self.number_with_unit(value, 1.0, 0)
        }
    }

    /// A MaxHit / MaxHeal amount (has its own unit switch and starts shortening at 1,000).
    pub fn strongest_action_amount_fragments(&self, value: f64) -> Vec<TextFragment> {
        if self.shorten_strongest_action_values && value >= MILLION {
            self.number_with_unit(value, MILLION, self.amount_decimals)
        } else if self.shorten_strongest_action_values && value >= THOUSAND {
            self.number_with_unit(value, THOUSAND, self.amount_decimals)
        } else {
            self.number_with_unit(value, 1.0, 0)
        }
    }

    /// A percentage with a dimmed "%" sign.
    pub fn percent_fragments(&self, value: f64) -> Vec<TextFragment> {
        vec![
            TextFragment::Plain(self.format_number(value, 1.0, self.percent_decimals)),
            TextFragment::Dimmed("%".into()),
        ]
    }
}

/// Inserts `separator` between groups of three digits ("1234567" -> "1,234,567").
fn group_thousands(integer_part: &str, separator: &str) -> String {
    let is_negative = integer_part.starts_with('-');
    let digits = integer_part.trim_start_matches('-');
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push_str(separator);
        }
        grouped.push(digit);
    }
    if is_negative { format!("-{grouped}") } else { grouped }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/formatting/number_format.rs"]
mod tests;
