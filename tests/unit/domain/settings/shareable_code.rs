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
