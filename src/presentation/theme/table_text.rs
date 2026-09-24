//! Text of the table rows: the local player's own row versus everybody else's.

use super::style_inputs::StyleInputs;
use std::fmt::Write;

pub(super) fn write(css: &mut String, inputs: &StyleInputs) {
    let font = inputs.font_stack("fBody", "'Segoe UI', 'sans-serif'");
    let size = inputs.slider_as_rem("sizeBodyText");
    let (bold_own, bold_other) = (inputs.bold_or_normal("boldYOU"), inputs.bold_or_normal("boldOther"));
    let _ = write!(
        css,
        ".tableWrap:not(#YOU):not(.myPet) .tableBody td{{color:#{};opacity:{};font-family:{font};font-weight:{bold_other};font-size:{size};text-shadow:{}}}\
         .tableWrap:not(#YOU):not(.myPet) .tableBody td .ex{{color:#{};opacity:{};text-shadow:{}}}\
         #YOU .tableBody td,.myPet .tableBody td{{color:#{};opacity:{};font-family:{font};font-weight:{bold_own};font-size:{size};text-shadow:{}}}\
         #YOU .tableBody td .ex,.myPet .tableBody td .ex{{color:#{};opacity:{};text-shadow:{}}}",
        inputs.color("tableOther"),
        inputs.opacity("tableOther"),
        inputs.text_outline("Other"),
        inputs.color("tableExOther"),
        inputs.opacity("tableOther"),
        inputs.text_outline("Other"),
        inputs.color("tableYOU"),
        inputs.opacity("tableYOU"),
        inputs.text_outline("YOU"),
        inputs.color("tableExYOU"),
        inputs.opacity("tableYOU"),
        inputs.text_outline("YOU"),
    );
}
