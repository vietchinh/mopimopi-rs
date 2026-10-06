//! Unit tests for `areas::bars`. They are compiled as a child module of that file but live here, outside `src/`.
//!
//! The bar colours and the fade must be exactly what the string-based `bar_color` and `with_optional_gradient` gave when they were recorded
//! (tests/fixtures/settings/bars/*.txt): a grid of palettes, jobs, roles and rows, and of fade directions.

use super::*;
use crate::domain::settings::Settings;
use serde_json::json;

fn recorded(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/settings/bars").join(format!("{name}.txt"))
}

/// A settings variant per palette and "my colour" choice, with the colours that make the rules visible (each key a different colour).
fn palette_variants() -> Vec<(String, Settings)> {
    let mut all = Vec::new();
    for palette in ["job", "role", "meYou", "something-else"] {
        for my_color in [false, true] {
            let mut settings = Settings::defaults();
            settings.set_option("palette", json!(palette));
            settings.set_option_enabled("myColorUse", my_color);
            for (index, key) in settings.file().colors.keys().cloned().collect::<Vec<_>>().into_iter().enumerate() {
                settings.set_color_hex(&key, &format!("{:06X}", 0x101010 + index * 0x030507));
            }
            all.push((format!("{palette}-my-{my_color}"), settings));
        }
    }
    all
}

const COLOR_KEYS: [&str; 12] = ["WAR", "WHM", "BLM", "SCH", "DRK", "LMB", "CBO", "pet", "ds", "oh", "Other", "nothing"];
const ROLE_KEYS: [&str; 7] = ["", "Tanker", "Healer", "Crafter", "Gathering", "DPS", "CBO"];
const ROW_IDS: [&str; 5] = ["YOU", "YOU_pet", "Other", "Aw_aern", "YOUR"];

fn grid(bars: &BarSettings) -> String {
    let mut lines = String::new();
    for color_key in COLOR_KEYS {
        for role_key in ROLE_KEYS {
            for row_id in ROW_IDS {
                lines.push_str(&format!("{color_key}|{role_key}|{row_id} {}\n", bars.palette.color_of(color_key, role_key, row_id)));
            }
        }
    }
    lines
}

fn fades() -> Vec<(String, Settings)> {
    let mut all = Vec::new();
    for on in [false, true] {
        for direction in ["top", "bottom", "left", "right", "diagonal", ""] {
            let mut settings = Settings::defaults();
            settings.set_option_enabled("gradient", on);
            settings.set_option("direction", json!(direction));
            all.push((format!("gradient-{on}-{direction}"), settings));
        }
    }
    all
}

#[test]
fn the_bar_colours_are_what_was_recorded() {
    for (name, settings) in palette_variants() {
        let bars = BarSettings::from_raw(settings.file());
        let expected = std::fs::read_to_string(recorded(&name)).unwrap_or_else(|_| panic!("{name}: nothing recorded; UPDATE_FIXTURES=1 cargo test record_the_old_bar_colours"));
        assert_eq!(grid(&bars), expected, "{name}");
    }
}

#[test]
fn the_fade_is_what_was_recorded() {
    let mut lines = String::new();
    for (name, settings) in fades() {
        lines.push_str(&format!("{name} {}\n", BarSettings::from_raw(settings.file()).fade.apply("#ABCDEF")));
    }
    assert_eq!(lines, std::fs::read_to_string(recorded("fades")).expect("recorded"));
}

#[test]
fn the_small_bars_float_to_the_side_of_their_own_table() {
    let mut settings = Settings::defaults();
    settings.set_option("bar_position_DPS", json!("right"));
    settings.set_option("bar_position", json!("left"));
    let bars = BarSettings::from_raw(settings.file());
    assert_eq!((bars.side(false), bars.side(true)), (Side::Right, Side::Left));
    assert_eq!((Side::Right.float_class(), Side::Left.float_class()), ("float-right", "float-left"));
    settings.set_option("bar_position", json!("anything else"));
    assert_eq!(BarSettings::from_raw(settings.file()).side(true), Side::Left);
}

#[test]
fn each_small_bar_and_the_animation_follow_their_switches() {
    let mut settings = Settings::defaults();
    for (key, on) in [("bar_pet", true), ("bar_ds", false), ("bar_oh", true), ("ani", false)] {
        settings.set_option_enabled(key, on);
    }
    let bars = BarSettings::from_raw(settings.file());
    assert_eq!((bars.pet, bars.shield, bars.overheal, bars.animate), (true, false, true, false));
}
