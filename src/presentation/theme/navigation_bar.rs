//! The top bar: background pattern, border, radius, icons and the time / target / summary text.

use super::color_conversion::rgba;
use super::style_inputs::{rem, StyleInputs};
use std::fmt::Write;

/// Layout choice "Display Type of Combatant Data": summary below the target (2 lines) or beside it.
const TWO_LINE_LAYOUT: i32 = 2;

pub(super) fn write(css: &mut String, inputs: &StyleInputs) {
    write_background_and_border(css, inputs);
    write_icons(css, inputs);
    write_texts(css, inputs);
    write_layout_spacing(css, inputs);
}

/// Background: plain colour or one of five patterns drawn over it.
fn background_rule(inputs: &StyleInputs) -> String {
    let pattern = rgba(&inputs.color("pattern"), inputs.opacity("pattern"));
    let base = rgba(&inputs.color("navBg"), inputs.opacity("navBg"));
    match inputs.text("pattern").as_str() {
        "cross" => format!(
            "-webkit-linear-gradient({pattern},transparent .1rem),-webkit-linear-gradient(0,{pattern},{base} .1rem)"
        ),
        "hStripe" => format!("-webkit-linear-gradient({pattern},transparent .1rem),-webkit-linear-gradient(0,{pattern},{base} 0)"),
        "vStripe" => format!("-webkit-linear-gradient({pattern},transparent 0),-webkit-linear-gradient(0,{pattern},{base} .1rem)"),
        "leftDig" => format!("repeating-linear-gradient(45deg,{pattern} 0,{pattern} 5%,{base} 0,{base} 50%) 0"),
        "rightDig" => format!("repeating-linear-gradient(135deg,{pattern} 0,{pattern} 5%,{base} 0,{base} 50%) 0"),
        _ => base,
    }
}

fn write_background_and_border(css: &mut String, inputs: &StyleInputs) {
    let pattern_size = inputs.slider_as_rem("sizePattern");
    let _ = write!(
        css,
        "nav[name=main],nav[name=history]{{background:{};height:{};background-size:{pattern_size} {pattern_size};background-repeat:repeat;{}}}",
        background_rule(inputs),
        inputs.slider_as_rem("sizeNav"),
        inputs.corner_radius_rules("rd_nav", "sizeRadius"),
    );
    if inputs.slider("navBg") != 100.0 {
        css.push_str(".btn_wrap{background:transparent}");
    }
    if inputs.slider("edge") != 0.0 {
        let edge_width = inputs.slider_as_rem("sizeEdge");
        let _ = write!(
            css,
            "nav[name=main],nav[name=history]{{border:{edge_width} {} {}}}\
             nav[name=main] .btn_wrap,nav[name=history] .btn_wrap{{right:{edge_width};top:{edge_width}}}\
             .previewArea nav[name=main] .btn_wrap{{right:0;top:0}}",
            inputs.text("edgeType"),
            inputs.color_with_opacity("edge", "edge"),
        );
    }
    let radius = inputs.slider("sizeRadius");
    let _ = write!(
        css,
        "nav[name=main] div[name=More]{{border-top-right-radius:{};border-bottom-right-radius:{}}}",
        rem(inputs.number("rd_navTR") * radius),
        rem(inputs.number("rd_navBR") * radius),
    );
}

fn write_icons(css: &mut String, inputs: &StyleInputs) {
    let _ = write!(
        css,
        "nav[name=main] i,nav[name=history] i{{color:{};font-size:{}}}",
        inputs.color_with_opacity("accent", "navIcon"),
        inputs.slider_as_rem("sizeIcon"),
    );
}

/// Padding that hides an element by giving it no size when its opacity slider is 0.
fn hidden_or_indented(opacity_slider_key: &str, inputs: &StyleInputs) -> &'static str {
    if inputs.slider(opacity_slider_key) == 0.0 { "padding-left:0;font-size:0" } else { "padding-left:1rem" }
}

fn write_texts(css: &mut String, inputs: &StyleInputs) {
    let _ = write!(
        css,
        "[name=time]{{color:{};font-family:{};font-size:{};font-style:{};{}}}",
        inputs.color_with_opacity("accent", "navTime"),
        inputs.font_stack("fTime", "'DS-Digital', 'sans-serif'"),
        inputs.slider_as_rem("sizeTime"),
        inputs.italic_or_normal("time_italic"),
        hidden_or_indented("navTime", inputs),
    );
    let _ = write!(
        css,
        "[name=target]{{color:{};font-family:{};font-size:{};font-style:{};{}}}",
        inputs.color_with_opacity("target", "target"),
        inputs.font_stack("fTarget", "'Segoe UI', 'sans-serif'"),
        inputs.slider_as_rem("sizeTarget"),
        inputs.italic_or_normal("target_italic"),
        hidden_or_indented("target", inputs),
    );
    let _ = write!(
        css,
        "[name=rps]{{color:{};font-family:{};font-size:{};font-style:{}}}",
        inputs.color_with_opacity("rps", "rps"),
        inputs.font_stack("fRPS", "'Roboto Condensed', 'Segoe UI', 'sans-serif'"),
        inputs.slider_as_rem("sizeRPS"),
        inputs.italic_or_normal("rps_italic"),
    );
}

fn write_layout_spacing(css: &mut String, inputs: &StyleInputs) {
    let gap = inputs.slider_as_rem("sizeGap");
    if inputs.number("act") as i32 == TWO_LINE_LAYOUT {
        let _ = write!(
            css,
            "[name=ACT_2line] [name=target]{{padding-bottom:{gap};vertical-align:bottom}}\
             [name=ACT_2line] [name=rps]{{padding-top:{gap};vertical-align:top}}"
        );
    } else {
        css.push_str(
            "[name=ACT_1line] [name=target]{padding-bottom:0;vertical-align:middle}\
             [name=ACT_1line] [name=rps]{padding-top:0;vertical-align:middle}",
        );
    }
}
