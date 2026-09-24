//! Table furniture: job icons, header, division lines and graph bar sizes.

use super::style_inputs::{rem, StyleInputs};
use std::fmt::Write;

/// (CSS class, slider key of its height) of every kind of graph bar.
const GRAPH_BARS: [(&str, &str); 4] =
    [("bar", "sizeGraph_bar"), ("pet", "sizeGraph_pet"), ("ds", "sizeGraph_ds"), ("oh", "sizeGraph_oh")];

pub(super) fn write(css: &mut String, inputs: &StyleInputs) {
    write_header_lines_and_icons(css, inputs);
    write_graph_bars(css, inputs);
    write_bar_positions_and_table_gaps(css, inputs);
}

fn write_header_lines_and_icons(css: &mut String, inputs: &StyleInputs) {
    let horizontal_line =
        format!("{} solid {}", inputs.slider_as_rem("sizeLine"), inputs.color_with_opacity("tableLine", "tableLine"));
    let header_color = inputs.color_with_opacity("tableHd", "tableHd");
    let vertical_line_width = inputs.slider_as_rem("sizeLineVer");
    let _ = write!(
        css,
        ".Class img{{width:{}}}\
         .tableBody td .ex{{font-size:{}}}\
         .tableHeader{{margin:{} 0}}\
         .tableHeader td{{background:{header_color};color:{};font-family:{};height:{};font-size:{};font-style:{}}}\
         .tableWrap{{border-bottom:{horizontal_line};height:{};margin-top:0}}\
         .barBg{{background:{}}}\
         .tableBody td:not(:last-child){{border-right:{vertical_line_width} solid {}}}\
         .tableHeader td:not(:last-child){{border-right:{vertical_line_width} solid {header_color}}}",
        inputs.slider_as_rem("sizeBodyIcon"),
        rem(inputs.slider("sizeBodyText") - 1.0),
        inputs.slider_as_rem("sizeHdGap"),
        inputs.color_with_opacity("tableHdText", "tableHdText"),
        inputs.font_stack("fHd", "'Roboto Condensed', 'sans-serif'"),
        inputs.slider_as_rem("sizeHd"),
        inputs.slider_as_rem("sizeHdText"),
        inputs.italic_or_normal("header_italic"),
        inputs.slider_as_rem("sizeBody"),
        inputs.color_with_opacity("tableBg", "tableBg"),
        inputs.color_with_opacity("tableLineVer", "tableLineVer"),
    );
}

fn write_graph_bars(css: &mut String, inputs: &StyleInputs) {
    let row_height = inputs.slider("sizeBody");
    for (bar_class, height_key) in GRAPH_BARS {
        let bar_height = inputs.slider(height_key);
        let _ = write!(
            css,
            ".tableWrap .{bar_class}{{height:{};margin-top:{};opacity:{}}}",
            rem(bar_height),
            rem(row_height - bar_height),
            inputs.opacity(bar_class),
        );
    }
}

fn write_bar_positions_and_table_gaps(css: &mut String, inputs: &StyleInputs) {
    let _ = write!(
        css,
        "#HPSBody .pet,#HPSBody .ds,#HPSBody .oh,#HPSBody_P .pet,#HPSBody_P .ds,#HPSBody_P .oh{{float:{}}}\
         #DPSBody .pet,#DPSBody_P .pet{{float:{}}}\
         #DPSHeader,#DPSHeader_P{{margin-top:{}}}#HPSHeader,#HPSHeader_P{{margin-top:{}}}",
        inputs.text("bar_position"),
        inputs.text("bar_position_DPS"),
        inputs.slider_as_rem("sizeDPSGap"),
        inputs.slider_as_rem("sizeHPSGap"),
    );
}
