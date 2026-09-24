//! Page-wide rules: root font size, background image and the accent colour.

use super::style_inputs::StyleInputs;
use std::fmt::Write;

pub(super) fn write(css: &mut String, inputs: &StyleInputs) {
    let background_image = if inputs.enabled("overlayBg") { inputs.text("overlayBgImg") } else { String::new() };
    let background_image_rule = if background_image.is_empty() {
        "none".to_string()
    } else {
        format!("url('{}')", background_image.replace('\'', "%27"))
    };
    let _ = write!(
        css,
        "html{{font-size:{};background-image:{};background-size:{};background-repeat:{}}}",
        inputs.text("resolution"),
        background_image_rule,
        inputs.text("overlayBgSize"),
        inputs.text("overlayBgRepeat"),
    );
    let accent = inputs.color("accent");
    let _ = write!(
        css,
        ".ac,.tab_title.on,nav[name=settings] i,input[type=range]{{color:#{accent}}}\
         .on_bar,.focus-border,.toggle{{background:#{accent}}}\
         .switch{{border-color:#{accent}}}\
         #preview24.on td:first-child{{color:#{accent}}}"
    );
}
