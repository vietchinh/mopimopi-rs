//! Unit tests for `areas::nav`. They are compiled as a child module of that file but live here, outside `src/`.
//!
//! The nav bar's variables must be exactly what the string-based `nav_bar_style` produced when they were recorded
//! (tests/fixtures/settings/nav-vars/*.txt, one per profile and per branch of the style), so the overlay looks the same.

use super::*;
use crate::domain::settings::Settings;

const PROFILES: [(&str, &str); 5] = [
    ("default", include_str!("../../../../../src/data/defaults.json")),
    ("header-only-bold-italic", include_str!("../../../../fixtures/settings/header-only-bold-italic.json")),
    ("body-only-gradient-left", include_str!("../../../../fixtures/settings/body-only-gradient-left.json")),
    ("both-gradient-bottom", include_str!("../../../../fixtures/settings/both-gradient-bottom.json")),
    ("raid-outline-gradient", include_str!("../../../../fixtures/settings/raid-outline-gradient.json")),
];

fn recorded(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/settings/nav-vars").join(format!("{name}.txt"))
}

/// Settings of the profile with the nav's own switches changed, to reach the branches the profiles do not (a pattern, an edge, hidden texts).
fn variants() -> Vec<(String, Settings)> {
    let mut all = Vec::new();
    for (name, text) in PROFILES {
        all.push((name.to_string(), Settings::from_json_text(text).unwrap()));
    }
    let base = Settings::from_json_text(PROFILES[0].1).unwrap();
    for pattern in ["cross", "hStripe", "vStripe", "leftDig", "rightDig", "none"] {
        let mut settings = base.clone();
        settings.set_option("pattern", serde_json::json!(pattern));
        all.push((format!("pattern-{pattern}"), settings));
    }
    let mut hidden = base.clone();
    hidden.set_slider_value("navTime", 0.0);
    hidden.set_slider_value("target", 0.0);
    all.push(("texts-hidden".into(), hidden));
    let mut no_edge = base.clone();
    no_edge.set_slider_value("edge", 0.0);
    no_edge.set_slider_value("navBg", 100.0);
    all.push(("no-edge-opaque".into(), no_edge));
    let mut one_line = base;
    one_line.set_option("act", serde_json::json!(1));
    one_line.set_option("time_italic", serde_json::json!(1));
    all.push(("one-line-italic".into(), one_line));
    all
}

/// `name:value` pairs of an inline style, in order.
fn declarations(vars: &str) -> Vec<(String, String)> {
    vars.split(';').filter(|part| !part.is_empty()).map(|part| part.split_once(':').map(|(name, value)| (name.trim_start_matches("--").to_string(), value.to_string())).unwrap_or_else(|| panic!("{part}"))).collect()
}

/// The recorded variables had the three texts' colour, font, size and style on the bar (`--nav-time-color`, ...). They are now the variables of
/// the cells that show each text (`--text-*`, with the values the nav texts do not have written out as what they were without a variable: full
/// opacity, normal weight, no text border); the bar keeps everything else, the spaces before the texts included, in the same order.
#[test]
fn the_nav_bar_variables_are_what_was_recorded_under_the_names_they_have_now() {
    for (name, settings) in variants() {
        let nav = NavSettings::from_raw(settings.file());
        let old = declarations(&std::fs::read_to_string(recorded(&name)).unwrap_or_else(|_| panic!("{name}: nothing recorded")));
        let value_of = |variable: &str| old.iter().find(|(n, _)| n == variable).unwrap_or_else(|| panic!("{name}: {variable} was not recorded")).1.clone();
        let belongs_to_a_text = |variable: &str| ["nav-time-", "nav-target-", "nav-rps-"].iter().any(|prefix| variable.starts_with(prefix) && !variable.ends_with("-padding"));
        let old_rest: Vec<_> = old.iter().filter(|(variable, _)| !belongs_to_a_text(variable)).cloned().collect();
        assert_eq!(declarations(&nav.bar_vars()), old_rest, "{name}: the bar");
        for (text, vars) in [("time", nav.time_vars()), ("target", nav.target_vars()), ("rps", nav.summary_vars())] {
            let new = declarations(&vars);
            let got = |variable: &str| new.iter().find(|(n, _)| n == variable).unwrap_or_else(|| panic!("{name}: {text}: no {variable}")).1.clone();
            for (suffix, variable) in [("color", "text-color"), ("font", "text-font"), ("size", "text-size"), ("style", "text-style")] {
                let recorded_value = value_of(&format!("nav-{text}-{suffix}"));
                // a hidden text had `0` for its size; it is `0rem` now, which is the same length
                let expected = if suffix == "size" && recorded_value == "0" { "0rem".to_string() } else { recorded_value };
                assert_eq!(got(variable), expected, "{name}: {text} {suffix}");
            }
            assert_eq!((got("text-opacity"), got("text-weight"), got("text-shadow")), ("1".to_string(), "normal".to_string(), "none".to_string()), "{name}: {text}");
            assert_eq!(new.len(), 7, "{name}: {text} has exactly its seven variables");
        }
    }
}

#[test]
fn the_summary_parts_and_the_pinned_buttons_follow_their_switches() {
    let mut settings = Settings::defaults();
    settings.set_option_enabled("act_rank", true);
    settings.set_option_enabled("act_max", false);
    settings.set_option_enabled("swap", true);
    settings.set_option_enabled("btn_History", true);
    settings.set_option_enabled("btn_Capture", false);
    let nav = NavSettings::from_raw(settings.file());
    assert!(nav.summary.rank && !nav.summary.max_hit && nav.summary.shows_strongest_heal);
    assert!(nav.pinned.history && !nav.pinned.capture);
}

#[test]
fn the_layout_is_two_lines_only_for_the_option_that_says_so() {
    let mut settings = Settings::defaults();
    for (option, two_lines) in [(1, false), (2, true), (3, false)] {
        settings.set_option("act", serde_json::json!(option));
        assert_eq!(NavSettings::from_raw(settings.file()).two_lines, two_lines, "act = {option}");
    }
}
