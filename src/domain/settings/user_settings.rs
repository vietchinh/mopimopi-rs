//! The settings document.

use serde_json::Value;

/// Section names of the settings JSON document.
pub(super) const OPTIONS_SECTION: &str = "q";
pub(super) const COLORS_SECTION: &str = "Color";
pub(super) const SLIDERS_SECTION: &str = "Range";
pub(super) const ABBREVIATIONS_SECTION: &str = "Alias";
pub(super) const COLUMN_ORDER_SECTION: &str = "Order";
pub(super) const COLUMN_DEFINITIONS_SECTION: &str = "ColData";

/// Settings sections that hold user-visible options and can be shared with others.
pub(super) const SHAREABLE_SECTIONS: [&str; 3] = [OPTIONS_SECTION, COLORS_SECTION, SLIDERS_SECTION];

/// Overlay settings, stored as the original overlay's JSON document.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub(super) json_document: Value,
}
