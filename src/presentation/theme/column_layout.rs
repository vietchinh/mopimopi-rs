//! Per-column width, padding and text alignment.

use super::style_inputs::{rem, StyleInputs};
use std::fmt::Write;

pub(super) fn write(css: &mut String, inputs: &StyleInputs) {
    let settings = inputs.settings;
    let body_font_style = inputs.italic_or_normal("body_italic");
    let header_font_style = inputs.italic_or_normal("header_italic");
    for table_label in ["DPS", "HPS"] {
        for column in settings.column_order(table_label) {
            let _ = write!(
                css,
                ".{column}.cell{{width:{};padding:0 {}}}\
                 .tableBody .{column}.cell{{text-align:{};font-style:{body_font_style}}}\
                 .tableHeader .{column}.cell{{text-align:{};font-style:{header_font_style}}}",
                rem(settings.column_number(&column, "width")),
                rem(settings.column_number(&column, "padding")),
                settings.column_text(&column, "alignBody"),
                settings.column_text(&column, "alignHeader"),
            );
        }
    }
    css.push_str(".name.cell{width:100%}"); // the name column takes the remaining width
}
