//! The value types. Each prints as the CSS it stands for.

use crate::domain::settings::Hex;
use crate::presentation::ui::shared::color_conversion::rgba;
use std::fmt::{self, Display, Write};

/// A size setting: tenths of a rem. Prints as a length (`"2.1rem"`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size(pub f64);

impl Size {
    pub const ZERO: Size = Size(0.0);
}

impl Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}rem", self.0 / 10.0)
    }
}

/// An opacity setting: a percentage. Prints as the fraction CSS wants (`0.5`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(pub f64);

impl Opacity {
    pub const FULL: Opacity = Opacity(100.0);

    pub fn fraction(self) -> f64 {
        self.0 / 100.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }
}

impl Display for Opacity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fraction().fmt(f)
    }
}

/// A colour. With an opacity it is joined with it, as the original did, into `rgba(r,g,b,a)`: a 3-digit colour uses each digit unscaled
/// there (`fff` is 15,15,15). Without one it is the hex colour itself (`#fff`, which CSS reads as white): the table texts are written
/// that way, with their opacity set separately, so that is kept.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    hex: Hex,
    opacity: Option<Opacity>,
}

impl Paint {
    pub fn with_opacity(hex: Hex, opacity: Opacity) -> Self {
        Paint { hex, opacity: Some(opacity) }
    }

    pub fn solid(hex: Hex) -> Self {
        Paint { hex, opacity: None }
    }
}

impl Display for Paint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.opacity {
            Some(opacity) => f.write_str(&rgba(self.hex.as_str(), opacity.fraction())),
            None => write!(f, "#{}", self.hex.as_str()),
        }
    }
}

/// A `font-family` list: the user's font first, then the original's fallbacks. The fallbacks quote `'sans-serif'` like the original did,
/// which makes browsers use their default font.
#[derive(Clone, Debug, PartialEq)]
pub struct FontStack(String);

impl FontStack {
    pub fn new(user_font: &str, fallbacks: &str) -> Self {
        FontStack(format!("'{user_font}', {fallbacks}"))
    }
}

impl Display for FontStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The `text-shadow` that gives text its border, in the "text border" colour: a soft glow, or an outline (four hard offsets).
#[derive(Clone, Debug, PartialEq)]
pub enum Shadow {
    /// No text border.
    None,
    Glow(Hex),
    Outline(Hex),
}

impl Display for Shadow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shadow::None => f.write_str("none"),
            Shadow::Outline(color) => {
                let color = color.as_str();
                write!(f, "-.1rem 0 #{color},0 .1rem #{color},.1rem 0 #{color},0 -.1rem #{color}")
            }
            Shadow::Glow(color) => write!(f, "0 0 .3rem #{}", color.as_str()),
        }
    }
}

/// Builds the `--name:value;` declarations of an inline `style`, in the order they are set. It replaces long `format!`s with positional
/// arguments, where swapping two arguments compiles and silently gives wrong colours or sizes.
#[derive(Default)]
pub struct Vars(String);

impl Vars {
    pub fn set(&mut self, name: &str, value: impl Display) -> &mut Self {
        let _ = write!(self.0, "--{name}:{value};");
        self
    }

    pub fn finish(self) -> String {
        self.0
    }
}
