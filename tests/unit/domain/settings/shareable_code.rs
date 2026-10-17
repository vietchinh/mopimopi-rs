//! Unit tests for `settings::shareable_code`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn shareable_code_round_trips_shared_values_only() {
    let mut source = Settings::defaults();
    source.set_slider_value("sizeBody", 30.0);
    let code = source.export_shareable_code();
    assert!(!code.contains("Lang"));
    let mut target = Settings::defaults();
    target.import_shareable_code(&code).unwrap();
    assert_eq!(target.slider_value("sizeBody"), 30.0);
    assert_eq!(target.import_shareable_code("nope"), Err(InvalidShareableCode));
}

// ---- recorded behaviour -----------------------------------------------------------------------------------------------
//
// What exporting a shareable code gives for complete settings files, and what importing that code on top of the defaults gives, recorded
// when the settings were still a loose JSON document (tests/fixtures/settings/exported/*.json and .../imported/*.json).
// Record again, on purpose, only when that is meant to change:  UPDATE_FIXTURES=1 cargo test shareable_code_matches

const FILES: [(&str, &str); 5] = [
    ("default", include_str!("../../../../src/data/defaults.json")),
    ("profile-header-only-bold-italic", include_str!("../../../fixtures/settings/header-only-bold-italic.json")),
    ("profile-body-only-gradient-left", include_str!("../../../fixtures/settings/body-only-gradient-left.json")),
    ("profile-both-gradient-bottom", include_str!("../../../fixtures/settings/both-gradient-bottom.json")),
    ("profile-raid-outline-gradient", include_str!("../../../fixtures/settings/raid-outline-gradient.json")),
];

fn recorded(kind: &str, name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/settings").join(kind).join(format!("{name}.json"))
}

fn check_or_record(kind: &str, name: &str, actual: &str) {
    let path = recorded(kind, name);
    if std::env::var("UPDATE_FIXTURES").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{kind}/{name}: nothing recorded at {}; record with UPDATE_FIXTURES=1", path.display()));
    assert_eq!(actual, expected, "{kind}/{name}");
}

#[test]
fn shareable_code_matches_what_was_recorded() {
    for (name, file) in FILES {
        let settings = Settings::from_json_text(file).unwrap();
        let code = settings.export_shareable_code();
        check_or_record("exported", name, &code);
        // the code applied to the defaults
        let mut imported = Settings::defaults();
        imported.import_shareable_code(&code).unwrap();
        check_or_record("imported", name, &imported.to_json_text());
    }
}

#[test]
fn a_shareable_code_that_is_not_settings_changes_nothing() {
    for code in ["", "not json", "[1,2]", "7", "null"] {
        let mut settings = Settings::defaults();
        assert_eq!(settings.import_shareable_code(code), Err(InvalidShareableCode), "{code:?}");
        assert_eq!(settings, Settings::defaults(), "{code:?}");
    }
}
