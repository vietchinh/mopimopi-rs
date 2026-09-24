//! Reads settings in the units the stylesheet needs.

use super::color_conversion::rgba;
use crate::domain::settings::Settings;

/// Sizes in the settings are stored in tenths of a rem.
const SIZE_UNITS_PER_REM: f64 = 10.0;
/// Slider opacities are percentages.
const PERCENT_PER_UNIT_OPACITY: f64 = 100.0;

pub(super) struct StyleInputs<'a> {
    pub settings: &'a Settings,
}

impl<'a> StyleInputs<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        StyleInputs { settings }
    }

    /// Raw slider value (`Range` section).
    pub fn slider(&self, key: &str) -> f64 {
        self.settings.slider_value(key)
    }

    /// Slider value as opacity between 0 and 1.
    pub fn opacity(&self, key: &str) -> f64 {
        self.slider(key) / PERCENT_PER_UNIT_OPACITY
    }

    /// Slider value as a rem length ("1.2rem").
    pub fn slider_as_rem(&self, key: &str) -> String {
        rem(self.slider(key))
    }

    /// Hex digits of a colour setting.
    pub fn color(&self, key: &str) -> String {
        self.settings.color_hex(key)
    }

    /// `rgba(...)` of colour `color_key` with the opacity slider `opacity_key`.
    pub fn color_with_opacity(&self, color_key: &str, opacity_key: &str) -> String {
        rgba(&self.color(color_key), self.opacity(opacity_key))
    }

    pub fn enabled(&self, key: &str) -> bool {
        self.settings.option_enabled(key)
    }

    pub fn number(&self, key: &str) -> f64 {
        self.settings.option_number(key)
    }

    pub fn text(&self, key: &str) -> String {
        self.settings.option_text(key)
    }

    pub fn italic_or_normal(&self, key: &str) -> &'static str {
        if self.enabled(key) { "italic" } else { "normal" }
    }

    pub fn bold_or_normal(&self, key: &str) -> &'static str {
        if self.enabled(key) { "bold" } else { "normal" }
    }

    /// `font-family` list: the user's font first, then the original's fallbacks. The fallbacks
    /// quote 'sans-serif' like the original did, which makes browsers use their default font.
    pub fn font_stack(&self, font_option_key: &str, fallbacks: &str) -> String {
        format!("'{}', {fallbacks}", self.text(font_option_key))
    }

    /// Four `border-*-radius` rules; `key_prefix` such as "rd_nav" selects the corners
    /// (`rd_navTL`, ...) and `size_key` the radius slider.
    pub fn corner_radius_rules(&self, key_prefix: &str, size_key: &str) -> String {
        let corner = |suffix: &str| rem(self.number(&format!("{key_prefix}{suffix}")) * self.slider(size_key));
        format!(
            "border-top-left-radius:{};border-top-right-radius:{};border-bottom-left-radius:{};border-bottom-right-radius:{}",
            corner("TL"),
            corner("TR"),
            corner("BL"),
            corner("BR")
        )
    }

    /// `text-shadow` that outlines text in the "text border" colour of `who` ("YOU" or "Other").
    pub fn text_outline(&self, who: &str) -> String {
        let color = self.color(&format!("tableBorder{who}"));
        if self.text("borderTextType") == "outline" {
            format!("-.1rem 0 #{color},0 .1rem #{color},.1rem 0 #{color},0 -.1rem #{color}")
        } else {
            format!("0 0 .3rem #{color}")
        }
    }
}

/// Converts a size setting (tenths of a rem) to a CSS length.
pub(super) fn rem(size: f64) -> String {
    format!("{}rem", size / SIZE_UNITS_PER_REM)
}
