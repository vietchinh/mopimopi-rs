//! Turns the settings into one CSS stylesheet.
//!
//! The original overlay re-applied ~300 lines of jQuery `.css()` calls after every render.
//! Here the same rules are produced once per settings change as a stylesheet, so any setting
//! restyles the overlay declaratively.
//!
//! * `style_inputs`      – helper that reads settings in CSS-friendly units
//! * `color_conversion`  – hex colours to `rgba(...)`
//! * `page_background`   – page font size, background image, accent colour
//! * `navigation_bar`    – top bar: background, border, icons, time / target / summary text
//! * `table_corners`     – corner radius of tables and graph bars
//! * `table_text`        – text of the table rows (own row vs. others)
//! * `raid_grid`         – the compact raid mode cards
//! * `table_chrome`      – header, lines, bars, icons
//! * `column_layout`     – per-column width, padding and alignment

mod color_conversion;
mod column_layout;
mod navigation_bar;
mod page_background;
mod raid_grid;
mod style_inputs;
mod table_chrome;
mod table_corners;
mod table_text;

use crate::domain::settings::Settings;
use style_inputs::StyleInputs;

const EXPECTED_STYLESHEET_SIZE_BYTES: usize = 16 * 1024;

/// The complete stylesheet for the current settings.
pub fn build_theme_css(settings: &Settings) -> String {
    let inputs = StyleInputs::new(settings);
    let mut css = String::with_capacity(EXPECTED_STYLESHEET_SIZE_BYTES);
    page_background::write(&mut css, &inputs);
    navigation_bar::write(&mut css, &inputs);
    table_corners::write(&mut css, &inputs);
    table_text::write(&mut css, &inputs);
    raid_grid::write(&mut css, &inputs);
    table_chrome::write(&mut css, &inputs);
    column_layout::write(&mut css, &inputs);
    css
}

/// Height in rem of a table body: rows are `sizeBody + sizeLine` tall, up to the configured row limit.
pub fn table_body_height_rem(settings: &Settings, table_label: &str, row_count: usize) -> f64 {
    let row_limit_key = if table_label == "HPS" { "sizeHPSTable" } else { "sizeDPSTable" };
    let row_limit = settings.slider_value(row_limit_key).max(0.0);
    let visible_rows = (row_count as f64).min(row_limit);
    visible_rows * (settings.slider_value("sizeBody") + settings.slider_value("sizeLine")) / 10.0
}

#[cfg(test)]
#[path = "../../../tests/unit/presentation/theme/mod.rs"]
mod tests;
