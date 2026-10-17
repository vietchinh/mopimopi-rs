//! Unit tests for `areas::raid`. They are compiled as a child module of that file but live here, outside `src/`.
//!
//! The grid's variables must be exactly what the string-based `grid_style` produced when they were recorded
//! (tests/fixtures/settings/raid-vars/*.txt), for every settings profile and for the variants that reach what the profiles do not.

use super::*;
use crate::domain::settings::Settings;
use serde_json::json;

const PROFILES: [(&str, &str); 5] = [
    ("default", include_str!("../../../../../src/data/defaults.json")),
    ("header-only-bold-italic", include_str!("../../../../fixtures/settings/header-only-bold-italic.json")),
    ("body-only-gradient-left", include_str!("../../../../fixtures/settings/body-only-gradient-left.json")),
    ("both-gradient-bottom", include_str!("../../../../fixtures/settings/both-gradient-bottom.json")),
    ("raid-outline-gradient", include_str!("../../../../fixtures/settings/raid-outline-gradient.json")),
];

fn variants() -> Vec<(String, Settings)> {
    let mut all: Vec<(String, Settings)> = PROFILES.iter().map(|(name, text)| (name.to_string(), Settings::from_json_text(text).unwrap())).collect();
    let mut add = |name: &str, change: &dyn Fn(&mut Settings)| {
        let mut settings = Settings::defaults();
        change(&mut settings);
        all.push((name.to_string(), settings));
    };
    add("sizes", &|s| {
        for (key, value) in [("size24TableSlice", 3.0), ("size24TableHeight", 31.0), ("size24BodyNameText", 14.0), ("size24BodyDataText", 11.0), ("size24TableIdxWd", 7.0), ("size24BodyIcon", 22.0), ("bar", 55.0)] {
            s.set_slider_value(key, value);
        }
    });
    add("no-cards-in-a-row", &|s| s.set_slider_value("size24TableSlice", 0.0));
    add("three-digit-colours", &|s| {
        s.set_color_hex("view24TableYOU", "f80");
        s.set_color_hex("view24BgOther", "0f0");
        s.set_color_hex("tableBorderOther", "00f");
    });
    add("bold-and-outline", &|s| {
        s.set_option_enabled("boldYOU", false);
        s.set_option_enabled("boldOther", true);
        s.set_option("borderTextType", json!("outline"));
        s.set_option_enabled("body_italic", true);
    });
    all
}

fn recorded(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/settings/raid-vars").join(format!("{name}.txt"))
}

/// `name:value` pairs of an inline style, in order.
fn declarations(vars: &str) -> Vec<(String, String)> {
    vars.split(';').filter(|part| !part.is_empty()).map(|part| part.split_once(':').map(|(name, value)| (name.trim_start_matches("--").to_string(), value.to_string())).unwrap_or_else(|| panic!("{part}"))).collect()
}

/// The recorded text variables were `--raid-text-font` and `-style` (shared) and `--raid-text-{other,own}-{weight,color,opacity,shadow}`. They are
/// now `--text-*` (the others) and `--own-*`, each with all six values; every recorded value must be there under its new name.
fn new_names(old: &str) -> Option<Vec<String>> {
    match old {
        "raid-text-font" => Some(vec!["text-font".into(), "own-font".into()]),
        "raid-text-style" => Some(vec!["text-style".into(), "own-style".into()]),
        _ => old.strip_prefix("raid-text-other-").map(|rest| vec![format!("text-{rest}")]).or_else(|| old.strip_prefix("raid-text-own-").map(|rest| vec![format!("own-{rest}")])),
    }
}

#[test]
fn the_grid_variables_are_what_was_recorded_under_the_names_they_have_now() {
    for (name, settings) in variants() {
        let new = declarations(&RaidSettings::from_raw(settings.file()).grid_vars());
        let old = declarations(&std::fs::read_to_string(recorded(&name)).unwrap_or_else(|_| panic!("{name}: nothing recorded")));
        let is_text = |variable: &str| variable.starts_with("text-") || variable.starts_with("own-");
        let new_rest: Vec<_> = new.iter().filter(|(variable, _)| !is_text(variable)).cloned().collect();
        let old_rest: Vec<_> = old.iter().filter(|(variable, _)| new_names(variable).is_none()).cloned().collect();
        assert_eq!(new_rest, old_rest, "{name}: what is not text");
        let mut expected_text: Vec<(String, String)> = old.iter().filter_map(|(variable, value)| new_names(variable).map(|news| news.into_iter().map(|n| (n, value.clone())).collect::<Vec<_>>())).flatten().collect();
        let mut new_text: Vec<(String, String)> = new.iter().filter(|(variable, _)| is_text(variable)).cloned().collect();
        expected_text.sort();
        new_text.sort();
        assert_eq!(new_text, expected_text, "{name}: text");
    }
}

#[test]
fn there_is_always_at_least_one_card_in_a_row() {
    for (setting, cards) in [(0.0, 1), (1.0, 1), (4.0, 4), (14.0, 14)] {
        let mut settings = Settings::defaults();
        settings.set_slider_value("size24TableSlice", setting);
        assert_eq!(RaidSettings::from_raw(settings.file()).cards_in_a_row(), cards, "slider at {setting}");
    }
}
