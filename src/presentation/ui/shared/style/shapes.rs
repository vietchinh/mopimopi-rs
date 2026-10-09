//! Shapes several areas have. Each knows the variable names it writes, so the names exist once and a variable cannot be given the wrong one.

use super::values::{FontStack, Opacity, Paint, Shadow, Size, Vars};
use std::fmt::{self, Display};

/// One themed text group (what `themed-text` in css/text.css shows): colour, opacity, the colour of the dimmed part (`.ex`), font, size, italic, bold and the text border.
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub color: Paint,
    /// Separate from the colour's own opacity: the nav texts bake it into the colour and leave this `FULL`, the table texts use it, which
    /// also dims the dimmed part and the text border.
    pub opacity: Opacity,
    /// The colour of the dimmed part; `None` when the text has none, and it then follows the colour.
    pub ex_color: Option<Paint>,
    pub font: FontStack,
    /// `None` when the cells set their own size (the raid cards' name and data) and the text inherits it otherwise.
    pub size: Option<Size>,
    pub italic: bool,
    pub bold: bool,
    pub shadow: Shadow,
}

impl TextStyle {
    /// Writes `--{prefix}-color`, `-opacity`, `-ex-color` and `-size` (when there are some), `-font`, `-style`, `-weight` and `-shadow`.
    pub fn write(&self, vars: &mut Vars, prefix: &str) {
        vars.set(&format!("{prefix}-color"), &self.color).set(&format!("{prefix}-opacity"), self.opacity);
        if let Some(ex_color) = &self.ex_color {
            vars.set(&format!("{prefix}-ex-color"), ex_color);
        }
        vars.set(&format!("{prefix}-font"), &self.font);
        if let Some(size) = self.size {
            vars.set(&format!("{prefix}-size"), size);
        }
        vars.set(&format!("{prefix}-style"), if self.italic { "italic" } else { "normal" })
            .set(&format!("{prefix}-weight"), if self.bold { "bold" } else { "normal" })
            .set(&format!("{prefix}-shadow"), &self.shadow);
    }
}

/// A line: `border`-style `width style colour` (`0.1rem solid rgba(...)`).
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub width: Size,
    pub style: String,
    pub paint: Paint,
}

impl Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.width, self.style, self.paint)
    }
}

/// The four corner radii of something: each corner's switch times the radius slider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corners {
    pub top_left: Size,
    pub top_right: Size,
    pub bottom_left: Size,
    pub bottom_right: Size,
}

impl Corners {
    /// No rounding at all (an area `applyScope` leaves out).
    pub const NONE: Corners = Corners { top_left: Size::ZERO, top_right: Size::ZERO, bottom_left: Size::ZERO, bottom_right: Size::ZERO };

    /// Writes `--{prefix}-tl`, `-tr`, `-bl` and `-br`.
    pub fn write(&self, vars: &mut Vars, prefix: &str) {
        vars.set(&format!("{prefix}-tl"), self.top_left)
            .set(&format!("{prefix}-tr"), self.top_right)
            .set(&format!("{prefix}-bl"), self.bottom_left)
            .set(&format!("{prefix}-br"), self.bottom_right);
    }

    /// As the value of `border-radius` (clockwise from the top left).
    pub fn border_radius(&self) -> String {
        format!("{} {} {} {}", self.top_left, self.top_right, self.bottom_right, self.bottom_left)
    }
}
