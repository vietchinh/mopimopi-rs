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
fn list_choices_keep_their_stored_type() {
    let mut settings = Settings::defaults();
    settings.set_option_from_text("dpsType", "1");
    assert!(settings.option_value("dpsType").is_number());
    settings.set_option_from_text("ds", ",");
    assert_eq!(settings.option_text("ds"), ",");
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
