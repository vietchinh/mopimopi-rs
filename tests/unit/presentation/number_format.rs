use super::*;

fn with_units() -> NumberFormat {
    NumberFormat { use_units: true, ..NumberFormat::default() }
}

fn european() -> NumberFormat {
    NumberFormat { decimal_separator: ',', thousands_separator: Some('.'), ..NumberFormat::default() }
}

fn with_rate_decimals() -> NumberFormat {
    NumberFormat { rate_decimals: 2, ..NumberFormat::default() }
}

#[test]
fn defaults_look_like_act() {
    let format = NumberFormat::default();
    assert_eq!(format.amount(1_943_397.0).to_string(), "1,943,397");
    assert_eq!(format.rate(6_106.61).to_string(), "6,107"); // ACT's ENCDPS
    assert_eq!(format.percent(Some(26.23)).to_string(), "26%");
}

/// The Zurvan log's encdps / ENCDPS pairs.
#[test]
fn rates_without_decimals_match_acts_rounded_fields() {
    let format = NumberFormat::default();
    assert_eq!(format.rate(1_076.26).to_string(), "1,076"); // Gwenn
    assert_eq!(format.rate(1_067.88).to_string(), "1,068"); // Kay
    assert_eq!(format.rate(119.79).to_string(), "120"); // Limit Break
    assert_eq!(format.rate(870.28).to_string(), "870"); // Hamtaro's HPS
}

#[test]
fn rates_with_decimals_match_acts_exact_fields() {
    assert_eq!(with_rate_decimals().rate(6_106.61).to_string(), "6,106.61");
}

#[test]
fn halves_round_away_from_zero_like_act() {
    let format = NumberFormat::default();
    assert_eq!(format.rate(812.5).to_string(), "813");
    assert_eq!(format.rate(2.5).to_string(), "3");
    assert_eq!(format.percent(Some(20.5)).to_string(), "21%");
    assert_eq!(format.amount(0.5).to_string(), "1");
}

/// Kay's crithit% and Gwenn's / Hamtaro's damage% / healed% from the Zurvan log.
#[test]
fn rates_round_but_shares_cut_off_like_act() {
    let format = NumberFormat::default();
    assert_eq!(format.percent(Some(20.75)).to_string(), "21%");
    assert_eq!(format.share(Some(17.62)).to_string(), "17%");
    assert_eq!(format.share(Some(41.94)).to_string(), "41%");

    let one_decimal = NumberFormat { percent_decimals: 1, ..NumberFormat::default() };
    assert_eq!(one_decimal.share(Some(17.629)).to_string(), "17.6%");
    assert_eq!(one_decimal.percent(Some(17.629)).to_string(), "17.6%");
    assert_eq!(one_decimal.percent(Some(17.66)).to_string(), "17.7%");
    assert_eq!(format.share(None).to_string(), "0%");
}

#[test]
fn units_like_mopimopi() {
    let format = with_units();
    assert_eq!(format.amount(1_943_397.0).to_string(), "1.9M");
    assert_eq!(format.amount(145_993.0).to_string(), "146.0k");
    assert_eq!(format.amount(9_999.0).to_string(), "9,999"); // "k" only from 10,000
    assert_eq!(format.rate(12_345.6).to_string(), "12.3k");
    assert_eq!(format.rate(2_000_000.0).to_string(), "2,000.0k"); // rates never use "M"
}

#[test]
fn unit_is_kept_apart_for_styling() {
    let formatted = with_units().amount(1_943_397.0);
    assert_eq!(formatted, FormattedNumber { number: "1.9".to_owned(), unit: Some("M") });
    assert_eq!(NumberFormat::default().percent(Some(21.05)).unit, Some("%"));
}

#[test]
fn other_separators() {
    let format = NumberFormat { rate_decimals: 2, ..european() };
    assert_eq!(format.amount(1_943_397.0).to_string(), "1.943.397");
    assert_eq!(format.rate(6_106.61).to_string(), "6.106,61");

    let no_grouping = NumberFormat { thousands_separator: None, ..NumberFormat::default() };
    assert_eq!(no_grouping.amount(1_943_397.0).to_string(), "1943397");
}

#[test]
fn infinity_and_missing_values() {
    let format = NumberFormat::default();
    assert_eq!(format.rate(f64::INFINITY).to_string(), "∞");
    assert_eq!(format.rate(f64::NAN).to_string(), "0");
    assert_eq!(format.amount(f64::NAN).to_string(), "0");
    assert_eq!(format.percent(None).to_string(), "0%");
}

#[test]
fn small_and_negative_numbers() {
    let format = NumberFormat::default();
    assert_eq!(format.amount(0.0).to_string(), "0");
    assert_eq!(format.amount(999.0).to_string(), "999");
    assert_eq!(format.amount(-1_234.0).to_string(), "-1,234");
    assert_eq!(with_rate_decimals().rate(0.5).to_string(), "0.50");
}
