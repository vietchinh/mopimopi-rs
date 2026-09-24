//! Raid mode: the compact grid of one small card per player.

use super::style_inputs::StyleInputs;
use std::fmt::Write;

pub(super) fn write(css: &mut String, inputs: &StyleInputs) {
    let line = format!("{} solid {}", inputs.slider_as_rem("sizeLine"), inputs.color_with_opacity("tableLine", "tableLine"));
    let font = inputs.font_stack("fBody", "'Segoe UI', 'sans-serif'");
    let (bold_own, bold_other) = (inputs.bold_or_normal("boldYOU"), inputs.bold_or_normal("boldOther"));
    let body_style = inputs.italic_or_normal("body_italic");
    let cards_per_row = inputs.slider("size24TableSlice").max(1.0);
    let _ = write!(
        css,
        ".rCell{{background:{};border-bottom:{line};height:{}}}\
         .rCell#YOU{{background:{}}}\
         .rCell td{{font-weight:{bold_other};color:#{};opacity:{};font-family:{font};font-style:{body_style};text-shadow:{}}}\
         .rCell#YOU td{{font-weight:{bold_own};color:#{};opacity:{};text-shadow:{}}}\
         .rName{{font-size:{}}}.rData{{font-size:{}}}\
         .rRow:first-child .rCell{{border-top:{line}}}\
         .rRow .rCell:last-child{{border-right:{line}}}\
         .rCell .rIdx{{width:{};opacity:{}}}\
         .rIcon,.rIcon img{{width:{}}}\
         .rRow{{display:grid;grid-template-columns:repeat({cards_per_row},1fr)}}\
         .rRow .rCell{{width:100%}}",
        inputs.color_with_opacity("view24BgOther", "view24BgOther"),
        inputs.slider_as_rem("size24TableHeight"),
        inputs.color_with_opacity("view24BgYOU", "view24BgYOU"),
        inputs.color("view24TableOther"),
        inputs.opacity("view24TableOther"),
        inputs.text_outline("Other"),
        inputs.color("view24TableYOU"),
        inputs.opacity("view24TableYOU"),
        inputs.text_outline("YOU"),
        inputs.slider_as_rem("size24BodyNameText"),
        inputs.slider_as_rem("size24BodyDataText"),
        inputs.slider_as_rem("size24TableIdxWd"),
        inputs.opacity("bar"),
        inputs.slider_as_rem("size24BodyIcon"),
    );
}
