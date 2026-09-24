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
        format.rate_fragments(12345.0),
        vec![TextFragment::Plain("12".into()), TextFragment::Dimmed("k".into())]
    );
}
