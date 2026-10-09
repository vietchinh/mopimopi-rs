//! The variable contract between the areas and the CSS.
//!
//! The names of the CSS variables are all that connects the Rust that sets them (each area's `*_vars`) and the CSS that reads them
//! (`css/*.css`). This builds every area from the settings profiles and checks both directions: nothing is read that nobody sets, and
//! nothing is set that nobody reads. A rename on one side only is a failure here, not a style that quietly stops applying.

use super::*;
use crate::domain::settings::Settings;
use serde_json::json;
use std::collections::BTreeSet;

const PROFILES: [&str; 5] = [
    include_str!("../../../../../src/data/defaults.json"),
    include_str!("../../../../fixtures/settings/header-only-bold-italic.json"),
    include_str!("../../../../fixtures/settings/body-only-gradient-left.json"),
    include_str!("../../../../fixtures/settings/both-gradient-bottom.json"),
    include_str!("../../../../fixtures/settings/raid-outline-gradient.json"),
];

/// The CSS that reads the variables.
const CSS: [(&str, &str); 6] = [
    ("nav.css", include_str!("../../../../../css/nav.css")),
    ("table.css", include_str!("../../../../../css/table.css")),
    ("raid.css", include_str!("../../../../../css/raid.css")),
    ("bars.css", include_str!("../../../../../css/bars.css")),
    ("text.css", include_str!("../../../../../css/text.css")),
    ("page.css", include_str!("../../../../../css/page.css")),
];

/// The Rust that reads variables itself: arbitrary classes (`not-last:[border-right:var(--chrome-cell-border)]`) and inline styles.
const RUST_THAT_READS: [(&str, &str); 8] = [
    ("standard_table.rs", include_str!("../../../../../src/presentation/ui/combat_tables/standard_table.rs")),
    ("raid_grid.rs", include_str!("../../../../../src/presentation/ui/combat_tables/raid_grid.rs")),
    ("history_screen/mod.rs", include_str!("../../../../../src/presentation/ui/history_screen/mod.rs")),
    ("history_row.rs", include_str!("../../../../../src/presentation/ui/history_screen/history_row.rs")),
    ("text_display.rs", include_str!("../../../../../src/presentation/ui/shared/text_display.rs")),
    ("buttons.rs", include_str!("../../../../../src/presentation/ui/navigation_bar/buttons.rs")),
    ("graph_bars.rs", include_str!("../../../../../src/presentation/ui/combat_tables/graph_bars.rs")),
    ("cell_classes.rs", include_str!("../../../../../src/presentation/ui/combat_tables/cell_classes.rs")),
];

/// Set per element where it is drawn, not by an area: the position a bar moves from (`graph_bars.rs`).
const SET_WHERE_DRAWN: [&str; 3] = ["bar-dx", "bar-sx", "bar-origin"];
/// Read with a fallback, so it does not have to be set: the text rules say what a text without a size, an italic, a weight, an opacity,
/// a border or a dimmed colour looks like.
const OPTIONAL: [&str; 6] = ["text-opacity", "text-size", "text-style", "text-weight", "text-shadow", "text-ex-color"];

/// Names a source reads: `var(--name)` in CSS and inline styles, and the utility classes of a variable, `w-(--name)` and `text-(length:--name)`.
fn read_by(source: &str) -> BTreeSet<String> {
    let without_comments = strip_comments(source);
    let mut names = BTreeSet::new();
    let bytes = without_comments.as_bytes();
    let mut at = 0;
    while let Some(found) = without_comments[at..].find("--") {
        let start = at + found;
        let before = if start > 0 { bytes[start - 1] } else { b' ' };
        let after = &without_comments[start + 2..];
        let end = after.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')).unwrap_or(after.len());
        // `(--name` is `var(--name` or a utility's `w-(--name)`; `:--name` is the type hint of `text-(length:--name)`. Anything else is a declaration.
        if (before == b'(' || before == b':') && end > 0 && !(before == b':' && !without_comments[..start].ends_with("length:") && !without_comments[..start].ends_with("color:")) {
            names.insert(after[..end].to_string());
        }
        at = start + 2 + end;
    }
    names
}

/// Variables a stylesheet defines itself (`text-own` swaps `--text-color` for `--own-color`).
fn defined_by(css: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in strip_comments(css).lines() {
        if let Some(rest) = line.trim_start().strip_prefix("--") {
            if let Some((name, _)) = rest.split_once(':') {
                names.insert(name.trim().to_string());
            }
        }
    }
    names
}

fn strip_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = rest[start..].find("*/").map_or("", |end| &rest[start + end + 2..]);
    }
    out.push_str(rest);
    out
}

/// The names in `--a:b;--c:d;` declarations (and in a `:root{...}` rule).
fn declared_in(style: &str) -> BTreeSet<String> {
    style.split(';').filter_map(|part| part.rsplit_once("--").map(|(_, rest)| rest)).filter_map(|rest| rest.split_once(':').map(|(name, _)| name.to_string())).collect()
}

/// Everything the areas set, over the profiles and the variants that reach the branches the profiles do not.
fn set_by_the_areas() -> BTreeSet<String> {
    let mut settings: Vec<Settings> = PROFILES.iter().map(|text| Settings::from_json_text(text).unwrap()).collect();
    let base = Settings::defaults();
    for pattern in ["cross", "hStripe", "vStripe", "leftDig", "rightDig"] {
        let mut variant = base.clone();
        variant.set_option("pattern", json!(pattern));
        settings.push(variant);
    }
    let mut no_edge = base.clone();
    no_edge.set_slider_value("edge", 0.0);
    settings.push(no_edge);
    let mut hidden = base.clone();
    hidden.set_slider_value("navTime", 0.0);
    settings.push(hidden);
    let mut image = base;
    image.set_option_enabled("overlayBg", true);
    image.set_option("overlayBgImg", json!("https://example.org/a.png"));
    settings.push(image);

    let mut names = BTreeSet::new();
    for settings in &settings {
        let file = settings.file();
        let (nav, table, raid, page) = (NavSettings::from_raw(file), TableSettings::from_raw(file), RaidSettings::from_raw(file), PageSettings::from_raw(file));
        for style in [nav.bar_vars(), nav.time_vars(), nav.target_vars(), nav.summary_vars(), table.header_vars(), table.body_vars(), raid.grid_vars(), page.root_rule()] {
            names.extend(declared_in(&style));
        }
    }
    names
}

#[test]
fn every_variable_the_css_reads_is_set_by_an_area_or_by_the_css_or_has_a_fallback() {
    let set = set_by_the_areas();
    let mut missing = Vec::new();
    for (file, css) in CSS {
        let defined = defined_by(css);
        for name in read_by(css) {
            let known = set.contains(&name) || defined.contains(&name) || SET_WHERE_DRAWN.contains(&name.as_str()) || OPTIONAL.contains(&name.as_str());
            if !known {
                missing.push(format!("{file}: --{name}"));
            }
        }
    }
    assert!(missing.is_empty(), "read by the CSS but never set: {missing:#?}");
}

#[test]
fn every_variable_an_area_sets_is_read_by_the_css() {
    let code_only = |source: &str| source.lines().filter(|line| !line.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
    let read: BTreeSet<String> = CSS.iter().flat_map(|(_, css)| read_by(css)).chain(RUST_THAT_READS.iter().flat_map(|(_, source)| read_by(&code_only(source)))).collect();
    // what is set for the markup to use, not the CSS: the page's own colour for inline styles (the colour picker reads its palette from the theme)
    let unread: Vec<String> = set_by_the_areas().into_iter().filter(|name| !read.contains(name)).map(|name| format!("--{name}")).collect();
    assert!(unread.is_empty(), "set by an area but read by no CSS: {unread:#?}");
}

#[test]
fn what_the_text_rules_swap_for_the_own_rows_is_what_the_areas_set() {
    let set = set_by_the_areas();
    for suffix in ["color", "opacity", "ex-color", "font", "size", "style", "weight", "shadow"] {
        assert!(set.contains(&format!("own-{suffix}")), "--own-{suffix}");
    }
}
