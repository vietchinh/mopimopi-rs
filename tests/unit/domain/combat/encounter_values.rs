use super::*;

#[test]
fn amount_never_holds_nan() {
    assert_eq!(Amount::new(f64::NAN).value(), 0.0);
    assert_eq!(Amount::new(12.5).value(), 12.5);
}

#[test]
fn rate_rounds_to_two_decimals_first_like_the_original() {
    assert_eq!(Rate::new(21.1538).value(), 21.15);
    assert_eq!(Rate::new(f64::NAN).value(), 0.0);
}

#[test]
fn rate_over_zero_seconds_is_infinity_or_zero() {
    assert_eq!(Rate::per_second(Amount::new(100.0), 0.0).value(), f64::INFINITY);
    assert_eq!(Rate::per_second(Amount::new(0.0), 0.0).value(), 0.0);
}

#[test]
fn rate_per_second_matches_a_known_pair() {
    // 1,943,397 damage over 318 s is the Zurvan sample's encdps, 6111.31 after pFloat.
    assert_eq!(Rate::per_second(Amount::new(1_943_397.0), 318.0).value(), 6111.31);
}

#[test]
fn percent_of_a_zero_whole_follows_javascript() {
    assert_eq!(Percent::of(Amount::new(5.0), Amount::new(0.0)).value(), f64::INFINITY);
    assert_eq!(Percent::of(Amount::new(0.0), Amount::new(0.0)).value(), 0.0);
    assert_eq!(Percent::of(Amount::new(1.0), Amount::new(3.0)).value(), 33.33);
}

#[test]
fn job_keeps_the_text_it_was_given() {
    assert_eq!(Job::from("Limit Break".to_string()).as_str(), "Limit Break");
    assert_eq!(Job::from(String::new()).to_string(), "");
}
