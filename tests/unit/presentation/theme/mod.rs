//! Unit tests for `theme::mod`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn stylesheet_contains_expected_rules() {
    let settings = Settings::defaults();
    let css = build_theme_css(&settings);
    assert!(css.contains(".rRow{display:grid;grid-template-columns:repeat(5,1fr)}"));
    assert!(css.contains(".tableWrap{border-bottom:0.1rem solid rgba(255,255,255,0.1);height:2.1rem;margin-top:0}"));
    assert!(css.contains(".encdps.cell{width:5rem;padding:0 0rem}"));
    assert_eq!(table_body_height_rem(&settings, "DPS", 20), 9.0 * 22.0 / 10.0);
}
