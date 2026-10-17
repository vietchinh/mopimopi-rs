//! Number formatting driven by the "Number" settings page.

use super::text_fragment::TextFragment;
use crate::domain::combat::{Amount, Percent, Rate, Reported};
use crate::domain::javascript_number::to_fixed;
use crate::domain::settings::{Settings, SettingKey};

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
    /// Show large `MaxHit` / `MaxHeal` values as "12.3k" / "1.2M".
    pub shorten_strongest_action_values: bool,
}

impl NumberFormat {
    pub fn from_settings(settings: &Settings) -> NumberFormat {
        let digits_after_decimal = settings.number(SettingKey::DecimalPlaces);
        let decimals_for = |kind_option: SettingKey| (digits_after_decimal * settings.number(kind_option)).max(0.0) as usize;
        let grouping = settings.text(SettingKey::ThousandsMark);
        let decimal_mark = settings.text(SettingKey::DecimalMark);
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
            rate_decimals: decimals_for(SettingKey::RateDecimals),
            percent_decimals: decimals_for(SettingKey::PercentDecimals),
            amount_decimals: decimals_for(SettingKey::AmountDecimals),
            shorten_large_values: settings.is_on(SettingKey::ShortenValues),
            shorten_strongest_action_values: settings.is_on(SettingKey::ShortenStrongestAction),
        }
    }

    /// `value / divisor` with grouping and decimal marks; non-finite values print as "0".
    pub fn format_number(&self, value: f64, divisor: f64, decimals: usize) -> String {
        if !value.is_finite() {
            return "0".into();
        }
        let fixed = to_fixed(value / divisor, decimals);
        // Written into one string: the grouped whole part, then the decimal mark and the fraction.
        let (integer_part, fraction) = match fixed.split_once('.') {
            Some((integer, fraction)) => (integer, Some(fraction)),
            None => (fixed.as_str(), None),
        };
        let mut formatted = String::with_capacity(fixed.len() + fixed.len() / 3 + self.decimal_separator.len());
        let (sign, digits) = integer_part.strip_prefix('-').map_or(("", integer_part), |digits| ("-", digits));
        formatted.push_str(sign);
        match &self.thousands_separator {
            Some(separator) => formatted.extend(grouped_by_three(digits, separator)),
            None => formatted.push_str(digits),
        }
        if let Some(fraction) = fraction {
            formatted.push_str(&self.decimal_separator);
            formatted.push_str(fraction);
        }
        formatted
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
    pub fn rate_fragments(&self, rate: Rate) -> Vec<TextFragment> {
        self.per_second_fragments(rate.value())
    }

    /// A per-second figure as ACT reported it.
    pub fn reported_rate_fragments(&self, rate: Reported) -> Vec<TextFragment> {
        self.per_second_fragments(rate.value())
    }

    fn per_second_fragments(&self, value: f64) -> Vec<TextFragment> {
        if value == f64::INFINITY {
            return vec![TextFragment::Plain("∞".into())];
        }
        let divisor = if self.shorten_large_values && value >= MINIMUM_FOR_THOUSANDS_UNIT { THOUSAND } else { 1.0 };
        self.number_with_unit(value, divisor, self.rate_decimals)
    }

    /// A damage / healing amount: k / M when large and enabled, otherwise a whole number.
    pub fn amount_fragments(&self, amount: Amount) -> Vec<TextFragment> {
        let value = amount.value();
        if self.shorten_large_values && value >= MILLION {
            self.number_with_unit(value, MILLION, self.amount_decimals)
        } else if self.shorten_large_values && value >= MINIMUM_FOR_THOUSANDS_UNIT {
            self.number_with_unit(value, THOUSAND, self.amount_decimals)
        } else {
            self.number_with_unit(value, 1.0, 0)
        }
    }

    /// A `MaxHit` / `MaxHeal` amount (has its own unit switch and starts shortening at 1,000).
    pub fn strongest_action_amount_fragments(&self, amount: Amount) -> Vec<TextFragment> {
        let value = amount.value();
        if self.shorten_strongest_action_values && value >= MILLION {
            self.number_with_unit(value, MILLION, self.amount_decimals)
        } else if self.shorten_strongest_action_values && value >= THOUSAND {
            self.number_with_unit(value, THOUSAND, self.amount_decimals)
        } else {
            self.number_with_unit(value, 1.0, 0)
        }
    }

    /// A percentage with a dimmed "%" sign.
    pub fn percent_fragments(&self, percent: Percent) -> Vec<TextFragment> {
        let value = percent.value();
        vec![
            TextFragment::Plain(self.format_number(value, 1.0, self.percent_decimals)),
            TextFragment::Dimmed("%".into()),
        ]
    }
}

/// The pieces of `digits` with `separator` between groups of three, counted from the right ("1234567" -> "1", ",", "234", ",", "567").
/// Pieces rather than a built string, so the caller writes them straight into the one string it is making.
fn grouped_by_three<'a>(digits: &'a str, separator: &'a str) -> impl Iterator<Item = &'a str> {
    let first_group = match digits.len() % 3 {
        0 => digits.len().min(3),
        remainder => remainder,
    };
    let (first, rest) = digits.split_at(first_group);
    std::iter::once(first).chain(rest.as_bytes().chunks(3).zip(0..).flat_map(move |(group, index)| {
        let start = first_group + index * 3;
        [separator, &digits[start..start + group.len()]]
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/formatting/number_format.rs"]
mod tests;
