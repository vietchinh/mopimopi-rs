//! Rounded corners of the tables and of the graph bars.

use super::style_inputs::{rem, StyleInputs};
use std::fmt::Write;

/// "Coverage" setting: which element the table's rounded corners are applied to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CornerCoverage {
    HeaderOnly,
    BodyOnly,
    HeaderAndBody,
}

impl CornerCoverage {
    fn from_option_number(number: i32) -> CornerCoverage {
        match number {
            1 => CornerCoverage::HeaderOnly,
            2 => CornerCoverage::BodyOnly,
            _ => CornerCoverage::HeaderAndBody,
        }
    }
}

pub(super) fn write(css: &mut String, inputs: &StyleInputs) {
    let radius = inputs.slider("sizeRadiusTable");
    let corner = |key: &str| rem(inputs.number(key) * radius);
    let left_corners = format!(
        "border-top-left-radius:{};border-bottom-left-radius:{}",
        corner("rd_tableTL"),
        corner("rd_tableBL")
    );
    let right_corners = format!(
        "border-top-right-radius:{};border-bottom-right-radius:{}",
        corner("rd_tableTR"),
        corner("rd_tableBR")
    );
    match CornerCoverage::from_option_number(inputs.number("applyScope") as i32) {
        CornerCoverage::HeaderOnly => {
            let _ = write!(css, ".tableHeader td:first-child{{{left_corners}}}.tableHeader td:last-child{{{right_corners}}}");
        }
        CornerCoverage::BodyOnly => {
            let _ = write!(css, ".barBg{{{left_corners};{right_corners}}}");
        }
        CornerCoverage::HeaderAndBody => {
            let _ = write!(
                css,
                ".tableHeader td:first-child,.barBg{{{left_corners}}}.tableHeader td:last-child,.barBg{{{right_corners}}}"
            );
        }
    }
    let _ = write!(css, ".bar,.mini div{{{}}}", inputs.corner_radius_rules("rd_graph", "sizeRadiusGraph"));
}
