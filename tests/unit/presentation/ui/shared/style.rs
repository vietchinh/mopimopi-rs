//! Unit tests for `shared::style`. They are compiled as a child module of that file but live here, outside `src/`.
//!
//! The values print as the CSS they stand for, the readers read the conventions of the file, and the original's quirks are kept on purpose.
//! (What the areas build from them is tested against what the string-based code gave, in `areas/`.)

use super::*;
use crate::domain::settings::{Hex, Settings};

const PROFILES: [(&str, &str); 5] = [
    ("default", include_str!("../../../../../src/data/defaults.json")),
    ("header-only-bold-italic", include_str!("../../../../fixtures/settings/header-only-bold-italic.json")),
    ("body-only-gradient-left", include_str!("../../../../fixtures/settings/body-only-gradient-left.json")),
    ("both-gradient-bottom", include_str!("../../../../fixtures/settings/both-gradient-bottom.json")),
    ("raid-outline-gradient", include_str!("../../../../fixtures/settings/raid-outline-gradient.json")),
];

/// Runs `check` once per profile, with the typed file and the profile's name.
fn for_each_profile(check: impl Fn(&str, &crate::domain::settings::SettingsFile)) {
    for (name, text) in PROFILES {
        let settings = Settings::from_json_text(text).unwrap_or_else(|| panic!("{name} loads"));
        check(name, settings.file());
    }
}

#[test]
fn choices_read_as_the_type_that_knows_them() {
    let expected = [("default", 3, "shadow"), ("header-only-bold-italic", 1, "outline"), ("body-only-gradient-left", 2, "shadow"), ("both-gradient-bottom", 3, "shadow"), ("raid-outline-gradient", 3, "outline")];
    for_each_profile(|name, file| {
        let (_, scope, border) = expected.iter().find(|(profile, ..)| *profile == name).unwrap();
        assert_eq!(file.choice::<i32>("applyScope"), *scope, "{name}");
        assert_eq!(file.choice::<String>("borderTextType"), *border, "{name}");
    });
}

// ---- the original's quirks, each kept on purpose ------------------------------------------------------------------------

#[test]
fn a_three_digit_colour_uses_each_digit_unscaled_when_it_is_joined_with_an_opacity_and_not_on_its_own() {
    assert_eq!(Paint::with_opacity(Hex::new("fff"), Opacity::FULL).to_string(), "rgba(15,15,15,1)");
    assert_eq!(Paint::solid(Hex::new("fff")).to_string(), "#fff", "CSS reads #fff as white");
    assert_eq!(Paint::with_opacity(Hex::new("03A9F4"), Opacity(50.0)).to_string(), "rgba(3,169,244,0.5)");
    assert_eq!(Paint::with_opacity(Hex::new("nonsense"), Opacity::FULL).to_string(), "rgba(0,0,0,1)", "something that is not a colour is black");
}

#[test]
fn the_fallback_fonts_keep_sans_serif_quoted() {
    assert_eq!(FontStack::new("Arial", "'Segoe UI', 'sans-serif'").to_string(), "'Arial', 'Segoe UI', 'sans-serif'");
}

#[test]
fn a_text_border_is_a_glow_or_an_outline_of_four_offsets() {
    assert_eq!(Shadow::Glow(Hex::new("000000")).to_string(), "0 0 .3rem #000000");
    assert_eq!(Shadow::Outline(Hex::new("FF0000")).to_string(), "-.1rem 0 #FF0000,0 .1rem #FF0000,.1rem 0 #FF0000,0 -.1rem #FF0000");
}

#[test]
fn sizes_are_tenths_of_a_rem_and_opacities_percentages() {
    for (size, css) in [(0.0, "0rem"), (5.0, "0.5rem"), (12.0, "1.2rem"), (21.0, "2.1rem"), (30.0, "3rem")] {
        assert_eq!(Size(size).to_string(), css);
    }
    assert_eq!(Opacity(75.0).to_string(), "0.75");
    assert_eq!(Opacity::FULL.to_string(), "1");
    assert!(Opacity(0.0).is_zero() && !Opacity(1.0).is_zero());
}

#[test]
fn no_corners_at_all_prints_zero_lengths() {
    assert_eq!(Corners::NONE.border_radius(), "0rem 0rem 0rem 0rem");
}

// ---- shapes and the variable builder ----------------------------------------------------------------------------------

#[test]
fn vars_writes_declarations_in_the_order_they_are_set() {
    let mut vars = Vars::default();
    vars.set("a", "1").set("b-c", Size(21.0)).set("d", Opacity(50.0));
    assert_eq!(vars.finish(), "--a:1;--b-c:2.1rem;--d:0.5;");
}

#[test]
fn a_text_style_writes_its_eight_variables_under_the_prefix_it_is_given() {
    let style = TextStyle {
        color: Paint::solid(Hex::new("FFFFFF")),
        opacity: Opacity(80.0),
        ex_color: Some(Paint::solid(Hex::new("BDBDBD"))),
        font: FontStack::new("Arial", "'sans-serif'"),
        size: Some(Size(12.0)),
        italic: true,
        bold: false,
        shadow: Shadow::Glow(Hex::new("000000")),
    };
    let mut vars = Vars::default();
    style.write(&mut vars, "table-text-own");
    assert_eq!(
        vars.finish(),
        "--table-text-own-color:#FFFFFF;--table-text-own-opacity:0.8;--table-text-own-ex-color:#BDBDBD;--table-text-own-font:'Arial', 'sans-serif';\
         --table-text-own-size:1.2rem;--table-text-own-style:italic;--table-text-own-weight:normal;--table-text-own-shadow:0 0 .3rem #000000;"
    );
}

#[test]
fn a_text_style_leaves_out_the_dimmed_colour_and_the_size_it_does_not_have() {
    let style = TextStyle {
        color: Paint::solid(Hex::new("FFFFFF")),
        opacity: Opacity::FULL,
        ex_color: None,
        font: FontStack::new("Arial", "'sans-serif'"),
        size: None,
        italic: false,
        bold: true,
        shadow: Shadow::None,
    };
    let mut vars = Vars::default();
    style.write(&mut vars, "text");
    assert_eq!(vars.finish(), "--text-color:#FFFFFF;--text-opacity:1;--text-font:'Arial', 'sans-serif';--text-style:normal;--text-weight:bold;--text-shadow:none;");
}

#[test]
fn a_line_prints_as_a_border_and_corners_write_the_names_they_are_given() {
    let mut vars = Vars::default();
    vars.set("chrome-row-border", Line { width: Size(2.0), style: "dashed".into(), paint: Paint::with_opacity(Hex::new("FF0000"), Opacity(50.0)) });
    Corners { top_left: Size(10.0), top_right: Size(20.0), bottom_left: Size(30.0), bottom_right: Size(40.0) }.write(&mut vars, "nav-radius");
    assert_eq!(vars.finish(), "--chrome-row-border:0.2rem dashed rgba(255,0,0,0.5);--nav-radius-tl:1rem;--nav-radius-tr:2rem;--nav-radius-bl:3rem;--nav-radius-br:4rem;");
}

/// Only in debug builds (`cargo test`, not `cargo test --release`): a mistyped key must not quietly give a default.
#[cfg(debug_assertions)]
mod mistyped_keys {
    use super::*;

    #[test]
    #[should_panic(expected = "unknown slider setting `sizeNavv`")]
    fn a_slider_that_does_not_exist_fails_loudly() {
        Settings::defaults().file().size("sizeNavv");
    }

    #[test]
    #[should_panic(expected = "unknown option `pet`")]
    fn an_option_that_does_not_exist_fails_loudly() {
        Settings::defaults().file().flag("pet");
    }

    #[test]
    #[should_panic(expected = "unknown colour setting `acent`")]
    fn a_colour_that_does_not_exist_fails_loudly() {
        Settings::defaults().file().solid("acent");
    }
}
