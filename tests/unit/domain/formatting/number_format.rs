//! Unit tests for `formatting::number_format`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

fn default_format() -> NumberFormat {
    NumberFormat::from_settings(&Settings::defaults())
}

#[test]
fn groups_and_rounds() {
    let format = default_format();
    assert_eq!(format.format_number(6703.94, 1.0, 0), "6,704");
    assert_eq!(format.format_number(6703.94, 1.0, 1), "6,703.9");
    assert_eq!(format.format_number(234638.0, 1000.0, 1), "234.6");
    assert_eq!(format.format_number(f64::INFINITY, 1.0, 0), "0");
}

#[test]
fn custom_separators() {
    let mut format = default_format();
    format.thousands_separator = Some(" ".into());
    format.decimal_separator = ",".into();
    assert_eq!(format.format_number(1234567.891, 1.0, 2), "1 234 567,89");
    format.thousands_separator = None;
    assert_eq!(format.format_number(1234567.0, 1.0, 0), "1234567");
}

#[test]
fn large_rates_are_shortened_when_enabled() {
    let format = default_format();
    assert_eq!(
        format.rate_fragments(Rate::new(12345.0)),
        vec![TextFragment::Plain("12".into()), TextFragment::Dimmed("k".into())]
    );
}

#[test]
fn a_rate_acts_reports_is_shown_as_reported_but_a_derived_one_is_rounded_twice_like_the_original() {
    let mut format = default_format();
    format.rate_decimals = 1;
    let shown = |fragments: Vec<TextFragment>| fragments.iter().map(|fragment| match fragment {
        TextFragment::Plain(text) | TextFragment::Dimmed(text) => text.clone(),
    }).collect::<String>();
    // 21.1538: shown to one decimal straight from ACT it is 21.2; the original's pFloat on a derived rate first makes it 21.15, then 21.1
    assert_eq!(shown(format.reported_rate_fragments(Reported::new(21.1538))), "21.2");
    assert_eq!(shown(format.rate_fragments(Rate::new(21.1538))), "21.1");
}

/// The grouping the app had before it wrote into one string: kept here as the reference the new code is held to.
fn grouped_the_old_way(integer_part: &str, separator: &str) -> String {
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

#[test]
fn grouping_in_pieces_gives_the_text_the_old_grouping_gave() {
    for separator in [",", ".", " ", "\u{202f}", "'"] {
        for length in 0..=24 {
            let digits: String = (0..length).map(|index| char::from(b'0' + (index % 10) as u8)).collect();
            for negative in [false, true] {
                let integer_part = if negative { format!("-{digits}") } else { digits.clone() };
                let mut written = String::new();
                if negative { written.push('-'); }
                written.extend(grouped_by_three(&digits, separator));
                assert_eq!(written, grouped_the_old_way(&integer_part, separator), "separator {separator:?}, {integer_part:?}");
            }
        }
    }
}
