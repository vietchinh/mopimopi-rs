//! Unit tests for `areas::page`. They are compiled as a child module of that file but live here, outside `src/`.

use super::*;
use crate::domain::settings::Settings;
use serde_json::json;

/// What `App` formatted for the page's variables before they were this struct's, written out again as the check.
fn root_rule_as_it_was(settings: &Settings) -> String {
    let image = if settings.option_enabled("overlayBg") { settings.option_text("overlayBgImg") } else { String::new() };
    let image_rule = if image.is_empty() { "none".to_string() } else { format!("url('{}')", image.replace('\'', "%27")) };
    format!(
        ":root{{--html-font-size:{};--html-bg-image:{};--html-bg-size:{};--html-bg-repeat:{};--accent:#{}}}",
        settings.option_text("resolution"),
        image_rule,
        settings.option_text("overlayBgSize"),
        settings.option_text("overlayBgRepeat"),
        settings.color_hex("accent"),
    )
}

#[test]
fn the_root_rule_is_what_the_page_always_had() {
    let mut with_image = Settings::defaults();
    with_image.set_option_enabled("overlayBg", true);
    with_image.set_option("overlayBgImg", json!("https://example.org/it's a picture.png"));
    with_image.set_option("resolution", json!("75%"));
    with_image.set_color_hex("accent", "FF4081");
    let mut image_off = with_image.clone();
    image_off.set_option_enabled("overlayBg", false);
    for (name, settings) in [("default", Settings::defaults()), ("with an image", with_image), ("image switched off", image_off)] {
        let rule = PageSettings::from_raw(settings.file()).root_rule();
        assert_eq!(rule.replace(";}", "}"), root_rule_as_it_was(&settings), "{name}");
    }
}

#[test]
fn a_quote_in_the_image_address_cannot_end_the_url() {
    let mut settings = Settings::defaults();
    settings.set_option_enabled("overlayBg", true);
    settings.set_option("overlayBgImg", json!("a'b"));
    assert!(PageSettings::from_raw(settings.file()).root_rule().contains("url('a%27b')"));
}

#[test]
fn the_language_falls_back_to_english() {
    let mut settings = Settings::defaults();
    settings.set_option("Lang", json!("KR"));
    assert_eq!(PageSettings::from_raw(settings.file()).language_code, "KR");
    settings.set_option("Lang", json!(""));
    assert_eq!(PageSettings::from_raw(settings.file()).language_code, "EN");
}

#[test]
fn the_page_switches_follow_their_settings() {
    let mut settings = Settings::defaults();
    for (key, on) in [("pets", false), ("tooltips", true), ("arrow", false)] {
        settings.set_option_enabled(key, on);
    }
    let page = PageSettings::from_raw(settings.file());
    assert_eq!((page.merge_pets, page.tooltips, page.corner_handle), (false, true, false));
}
