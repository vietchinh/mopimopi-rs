//! Unit tests for `areas::table`. They are compiled as a child module of that file but live here, outside `src/`.
//!
//! The header and body variables, and the height of a body, must be exactly what the string-based `header_style`, `body_style` and
//! `table_body_height_rem` produced when they were recorded (tests/fixtures/settings/table-vars/), for every settings profile and for
//! the variants that reach what the profiles do not: the overlay has to look the same.

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
    let base = Settings::defaults();
    let mut add = |name: &str, change: &dyn Fn(&mut Settings)| {
        let mut settings = base.clone();
        change(&mut settings);
        all.push((name.to_string(), settings));
    };
    for scope in [1, 2, 3] {
        add(&format!("scope-{scope}"), &|s| s.set_option("applyScope", json!(scope)));
    }
    add("bold-own-only", &|s| {
        s.set_option_enabled("boldYOU", true);
        s.set_option_enabled("boldOther", false);
    });
    add("bold-other-only", &|s| {
        s.set_option_enabled("boldYOU", false);
        s.set_option_enabled("boldOther", true);
    });
    add("italic-everywhere", &|s| {
        for key in ["header_italic", "body_italic"] {
            s.set_option_enabled(key, true);
        }
    });
    add("no-radii", &|s| {
        for key in ["rd_tableTL", "rd_tableTR", "rd_tableBL", "rd_tableBR", "rd_graphTL", "rd_graphTR", "rd_graphBL", "rd_graphBR"] {
            s.set_option_enabled(key, false);
        }
    });
    add("bar-sizes", &|s| {
        for (key, value) in [("sizeGraph_bar", 5.0), ("sizeGraph_pet", 3.0), ("sizeGraph_ds", 2.0), ("sizeGraph_oh", 4.0), ("bar", 60.0), ("pet", 40.0), ("ds", 25.0), ("oh", 10.0)] {
            s.set_slider_value(key, value);
        }
    });
    add("row-limits", &|s| {
        for (key, value) in [("sizeDPSTable", 3.0), ("sizeHPSTable", 7.0), ("sizeLine", 2.0), ("sizeBody", 24.0)] {
            s.set_slider_value(key, value);
        }
    });
    add("three-digit-colours", &|s| {
        // a 3-digit colour is used unscaled when it is joined with an opacity and as CSS's own #hex when it is not
        s.set_color_hex("tableYOU", "f80");
        s.set_color_hex("tableHd", "0f0");
        s.set_color_hex("tableBorderOther", "00f");
    });
    all
}

fn recorded(name: &str, part: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/settings/table-vars").join(format!("{name}.{part}.txt"))
}

const ROW_COUNTS: [usize; 5] = [0, 1, 3, 9, 30];

/// What the body height is for both tables and a few row counts, one line each.
fn heights(settings: &Settings, height: impl Fn(&Settings, bool, usize) -> f64) -> String {
    let mut lines = String::new();
    for is_healing in [false, true] {
        for count in ROW_COUNTS {
            lines.push_str(&format!("{is_healing} {count} {}\n", height(settings, is_healing, count)));
        }
    }
    lines
}

fn read(name: &str, part: &str) -> String {
    std::fs::read_to_string(recorded(name, part)).unwrap_or_else(|_| panic!("{name}.{part}: nothing recorded; UPDATE_FIXTURES=1 cargo test record_the_old_table_styles"))
}

/// `name:value` pairs of an inline style, in order.
fn declarations(vars: &str) -> Vec<(String, String)> {
    vars.split(';').filter(|part| !part.is_empty()).map(|part| part.split_once(':').map(|(name, value)| (name.trim_start_matches("--").to_string(), value.to_string())).unwrap_or_else(|| panic!("{part}"))).collect()
}

/// The recorded body variables were written when the text of own and other rows had a set of variables each, `--table-text-own-*` and
/// `--table-text-other-*`, and the font, size and italic were shared. They are now `--own-*` and `--text-*` (the other rows'), each with all
/// eight values; every recorded value must be there under its new name, and everything else must be unchanged and in the same order.
fn old_text_names_to_new(name: &str) -> Option<Vec<String>> {
    let both = |suffix: &str| Some(vec![format!("text-{suffix}"), format!("own-{suffix}")]);
    match name {
        "table-text-font" => both("font"),
        "table-text-size" => both("size"),
        "column-body-style" => both("style"),
        _ => {
            for (old, new) in [("table-text-own-", "own-"), ("table-text-other-", "text-")] {
                if let Some(suffix) = name.strip_prefix(old) {
                    return Some(vec![format!("{new}{suffix}")]);
                }
            }
            None
        }
    }
}

/// The header's text was `--chrome-header-color`, `-font`, `-text-size` and `--column-header-style`. It is `--text-*` now, with the three values
/// the header does not have written out as what they were without a variable: full opacity, normal weight, no text border.
#[test]
fn the_header_variables_are_what_was_recorded_under_the_names_they_have_now() {
    for (name, settings) in variants() {
        let new = declarations(&TableSettings::from_raw(settings.file()).header_vars());
        let old = declarations(&read(&name, "header"));
        let renamed = |variable: &str| match variable {
            "chrome-header-color" => Some("text-color"),
            "chrome-header-font" => Some("text-font"),
            "chrome-header-text-size" => Some("text-size"),
            "column-header-style" => Some("text-style"),
            _ => None,
        };
        let new_rest: Vec<_> = new.iter().filter(|(variable, _)| !variable.starts_with("text-")).cloned().collect();
        let old_rest: Vec<_> = old.iter().filter(|(variable, _)| renamed(variable).is_none()).cloned().collect();
        assert_eq!(new_rest, old_rest, "{name}: what is not text");
        let mut expected_text: Vec<(String, String)> = old.iter().filter_map(|(variable, value)| renamed(variable).map(|new| (new.to_string(), value.clone()))).collect();
        expected_text.extend([("text-opacity", "1"), ("text-weight", "normal"), ("text-shadow", "none")].map(|(variable, value)| (variable.to_string(), value.to_string())));
        let mut new_text: Vec<(String, String)> = new.iter().filter(|(variable, _)| variable.starts_with("text-")).cloned().collect();
        expected_text.sort();
        new_text.sort();
        assert_eq!(new_text, expected_text, "{name}: text");
    }
}

#[test]
fn the_body_variables_are_what_was_recorded_under_the_names_they_have_now() {
    for (name, settings) in variants() {
        let new = declarations(&TableSettings::from_raw(settings.file()).body_vars());
        let old = declarations(&read(&name, "body"));
        let is_text = |variable: &str| variable.starts_with("text-") || variable.starts_with("own-");
        let new_rest: Vec<_> = new.iter().filter(|(variable, _)| !is_text(variable)).cloned().collect();
        let old_rest: Vec<_> = old.iter().filter(|(variable, _)| old_text_names_to_new(variable).is_none()).cloned().collect();
        assert_eq!(new_rest, old_rest, "{name}: what is not text");
        let mut expected_text: Vec<(String, String)> = old.iter().filter_map(|(variable, value)| old_text_names_to_new(variable).map(|news| news.into_iter().map(|n| (n, value.clone())).collect::<Vec<_>>())).flatten().collect();
        let mut new_text: Vec<(String, String)> = new.iter().filter(|(variable, _)| is_text(variable)).cloned().collect();
        expected_text.sort();
        new_text.sort();
        assert_eq!(new_text, expected_text, "{name}: text");
    }
}

#[test]
fn the_body_height_is_what_was_recorded() {
    for (name, settings) in variants() {
        let table = TableSettings::from_raw(settings.file());
        let new = |_: &Settings, is_healing: bool, count: usize| table.body_height_rem(is_healing, count);
        assert_eq!(heights(&settings, new), read(&name, "heights"), "{name}");
    }
}

#[test]
fn the_tables_are_drawn_in_the_order_and_combination_the_user_chose() {
    let mut settings = Settings::defaults();
    let table = |settings: &Settings| TableSettings::from_raw(settings.file());
    for (order, dps, hps, expected) in [
        (1, true, true, vec![false, true]),
        (2, true, true, vec![true, false]),
        (1, true, false, vec![false]),
        (2, true, false, vec![false]),
        (2, false, true, vec![true]),
        (1, false, false, vec![]),
    ] {
        settings.set_option("tableOrder", json!(order));
        settings.set_option_enabled("viewDPS", dps);
        settings.set_option_enabled("viewHPS", hps);
        assert_eq!(table(&settings).tables_in_order(), expected, "order {order}, DPS {dps}, HPS {hps}");
    }
}

#[test]
fn raid_mode_needs_the_switch_and_enough_players() {
    let mut settings = Settings::defaults();
    settings.set_option_enabled("view24", true);
    settings.set_option("view24_Number", json!(14));
    let table = TableSettings::from_raw(settings.file());
    assert!(!table.raid_mode(13) && table.raid_mode(14) && table.raid_mode(24));
    settings.set_option_enabled("view24", false);
    assert!(!TableSettings::from_raw(settings.file()).raid_mode(24), "switched off");
}

#[test]
fn each_table_has_its_own_job_filter_gap_and_row_limit() {
    let mut settings = Settings::defaults();
    for (key, on) in [("DPS_T", true), ("DPS_H", false), ("DPS_D", true), ("DPS_C", false), ("DPS_M", true), ("HPS_T", false), ("HPS_H", true), ("HPS_D", false), ("HPS_C", true), ("HPS_M", false)] {
        settings.set_option_enabled(key, on);
    }
    settings.set_slider_value("sizeDPSGap", 4.0);
    settings.set_slider_value("sizeHPSGap", 9.0);
    settings.set_slider_value("sizeDPSTable", 6.0);
    settings.set_slider_value("sizeHPSTable", -2.0);
    let table = TableSettings::from_raw(settings.file());
    assert_eq!(table.damage.filter, JobFilter { tanks: true, healers: false, damage_dealers: true, chocobos: false, crafters_and_gatherers: true });
    assert_eq!(table.healing.filter, JobFilter { tanks: false, healers: true, damage_dealers: false, chocobos: true, crafters_and_gatherers: false });
    assert_eq!((table.damage.gap.to_string(), table.healing.gap.to_string()), ("0.4rem".to_string(), "0.9rem".to_string()));
    assert_eq!((table.damage.row_limit, table.healing.row_limit), (6.0, 0.0), "a negative limit shows no rows");
}

#[test]
fn corners_follow_apply_scope() {
    for (scope, header, body) in [(1, true, false), (2, false, true), (3, true, true), (7, true, true)] {
        let mut settings = Settings::defaults();
        settings.set_option("applyScope", json!(scope));
        settings.set_slider_value("sizeRadiusTable", 10.0);
        let table = TableSettings::from_raw(settings.file());
        assert_eq!((table.header.corners != Corners::NONE, table.body.corners != Corners::NONE), (header, body), "applyScope {scope}");
    }
}
