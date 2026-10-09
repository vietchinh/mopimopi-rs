//! Unit tests for `settings::persistence`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn defaults_have_expected_values() {
    let settings = Settings::defaults();
    assert_eq!(settings.option_number("mhh"), 2.0);
    assert!(settings.option_enabled("pets"));
    assert_eq!(settings.color_hex("accent"), "03A9F4");
    assert_eq!(settings.column_order("DPS")[2], "encdps");
}

#[test]
fn restores_from_json_text_and_rejects_garbage() {
    let text = Settings::defaults().to_json_text();
    assert!(Settings::from_json_text(&text).is_some());
    assert!(Settings::from_json_text("nope").is_none());
}

#[test]
fn list_choices_are_stored_as_text_like_the_original_does() {
    let mut settings = Settings::defaults();
    settings.set_option_from_text("dpsType", "1");
    // the original stores the picked entry's key as text ("1"), and every reader accepts that
    assert_eq!(settings.option_value("dpsType"), serde_json::json!("1"));
    assert_eq!(settings.option_number("dpsType"), 1.0);
    settings.set_option_from_text("ds", ",");
    assert_eq!(settings.option_text("ds"), ",");
}

#[test]
fn a_table_shows_the_ordered_columns_whose_flag_is_on() {
    let mut settings = Settings::defaults();
    let ordered = settings.column_order("DPS");
    // what the settings screens write keeps `Order` and the flags in step, and so do the defaults
    assert_eq!(settings.file().enabled_columns("DPS"), ordered.iter().map(String::as_str).collect::<Vec<_>>());
    // a column that is still in `Order` but whose flag is off (a hand-edited file can look like this) is not shown, by header or body
    let switched_off = ordered[1].clone();
    settings.set_column_field(&switched_off, "DPS", serde_json::json!(0));
    let shown = settings.file().enabled_columns("DPS");
    assert!(!shown.contains(&switched_off.as_str()));
    assert_eq!(shown.len(), ordered.len() - 1);
    assert!(settings.column_order("DPS").contains(&switched_off), "Order itself is untouched");
}

#[test]
fn columns_can_be_toggled_and_moved() {
    let mut settings = Settings::defaults();
    settings.set_column_enabled("duration", "DPS", true);
    assert!(settings.column_order("DPS").contains(&"duration".to_string()));
    settings.set_column_enabled("duration", "DPS", false);
    assert!(!settings.column_order("DPS").contains(&"duration".to_string()));
    settings.move_column("name", "DPS", false);
    assert_eq!(settings.column_order("DPS")[1], "encdps");
}

// ---- loading: recorded behaviour ------------------------------------------------------------------------------------
//
// What loading a settings file produces for a set of files, recorded when the settings were still a loose JSON document
// (tests/fixtures/settings/loaded/*.json) and kept as the definition of "the same" ever since. Record again, on purpose,
// only when loading is meant to behave differently:  UPDATE_FIXTURES=1 cargo test loading_reproduces

/// Files that are complete settings documents: loading them changes nothing, not even the order of the keys.
const COMPLETE_FILES: [(&str, &str); 5] = [
    ("default", include_str!("../../../../src/data/defaults.json")),
    ("profile-header-only-bold-italic", include_str!("../../../fixtures/settings/header-only-bold-italic.json")),
    ("profile-body-only-gradient-left", include_str!("../../../fixtures/settings/body-only-gradient-left.json")),
    ("profile-both-gradient-bottom", include_str!("../../../fixtures/settings/both-gradient-bottom.json")),
    ("profile-raid-outline-gradient", include_str!("../../../fixtures/settings/raid-outline-gradient.json")),
];
/// Older, partial and extended files: loading fills in, cleans up and keeps what it does not know.
const OTHER_FILES: [(&str, &str); 3] = [
    ("older-version", include_str!("../../../fixtures/settings/import/older-version.json")),
    ("no-columns", include_str!("../../../fixtures/settings/import/no-columns.json")),
    ("only-options", include_str!("../../../fixtures/settings/import/only-options.json")),
];

fn recorded_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/settings/loaded").join(format!("{name}.json"))
}

/// The same JSON as compact text, key order untouched.
fn compact(json: &str) -> String {
    serde_json::to_string(&serde_json::from_str::<serde_json::Value>(json).unwrap()).unwrap()
}

#[test]
fn loading_reproduces_what_was_recorded() {
    let record = std::env::var("UPDATE_FIXTURES").is_ok();
    for (name, input) in COMPLETE_FILES.iter().chain(OTHER_FILES.iter()) {
        let saved = Settings::from_json_text(input).unwrap_or_else(|| panic!("{name}: was not loaded")).to_json_text();
        let path = recorded_path(name);
        if record {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &saved).unwrap();
            continue;
        }
        let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{name}: nothing recorded at {}; record with UPDATE_FIXTURES=1", path.display()));
        if COMPLETE_FILES.iter().any(|(complete, _)| complete == name) {
            assert_eq!(saved, expected, "{name}: loading and saving no longer gives what was recorded");
        } else {
            // A file that was filled in or repaired is compared section by section, key order inside each section included. Only the order of
            // the sections themselves is not compared: the typed file always writes them in the original's order (q, Color, Range, Alias,
            // Order, ColData), where the loose document appended a missing one at the end.
            let (saved, expected): (serde_json::Value, serde_json::Value) = (serde_json::from_str(&saved).unwrap(), serde_json::from_str(&expected).unwrap());
            let keys = |document: &serde_json::Value| document.as_object().unwrap().keys().cloned().collect::<std::collections::BTreeSet<_>>();
            assert_eq!(keys(&saved), keys(&expected), "{name}: sections");
            for section in keys(&saved) {
                assert_eq!(serde_json::to_string(&saved[&section]).unwrap(), serde_json::to_string(&expected[&section]).unwrap(), "{name}: section {section}");
            }
        }
    }
}

#[test]
fn a_complete_file_is_saved_exactly_as_it_was_read() {
    for (name, input) in COMPLETE_FILES {
        let saved = Settings::from_json_text(input).unwrap().to_json_text();
        assert_eq!(saved, compact(input), "{name}: values, number formats or key order changed");
    }
}

#[test]
fn an_older_file_is_brought_up_to_date_and_nothing_it_has_is_lost() {
    let saved: serde_json::Value = serde_json::from_str(&Settings::from_json_text(OTHER_FILES[0].1).unwrap().to_json_text()).unwrap();
    // filled in from the defaults
    assert!(saved["Color"].get("VPR").is_some() && saved["Color"].get("PCT").is_some(), "colours added by newer versions");
    assert!(saved["q"].get("resolution").is_some() && saved["q"].get("view24").is_some());
    // the old version's "Cell" keys are gone, the outdated raid threshold is the current one
    assert!(saved["q"].get("tableCellOld").is_none() && saved["Color"].get("tableCellColor").is_none() && saved["Range"].get("sizeSomethingCellHeight").is_none());
    assert_eq!(saved["q"]["view24_Number"], serde_json::json!(14));
    // what this version does not know is kept
    assert_eq!(saved["q"]["aKeyFromTheFuture"], serde_json::json!(7));
    assert_eq!(saved["Future"]["values"], serde_json::json!([1, 2, 3]));
}

#[test]
fn a_file_without_column_settings_gets_the_default_columns() {
    let loaded = Settings::from_json_text(OTHER_FILES[1].1).unwrap();
    assert_eq!(loaded.column_order("DPS"), Settings::defaults().column_order("DPS"));
    assert_eq!(loaded.column_number("name", "padding"), Settings::defaults().column_number("name", "padding"));
}

#[test]
fn something_that_is_not_a_settings_object_is_not_loaded() {
    for text in ["[1,2]", "7", "\"text\"", "null", "{not json"] {
        assert!(Settings::from_json_text(text).is_none(), "{text}");
    }
}
