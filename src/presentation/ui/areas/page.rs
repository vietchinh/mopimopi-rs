//! The page as a whole: the language, the font size and background of the page, the accent colour, and the switches that are not about one
//! area (merging pets into their owners, tooltips, the corner handle).

use crate::domain::settings::{Hex, SettingsFile};
use crate::presentation::ui::shared::style::{StyleReads, Vars};

#[derive(Clone, Debug, PartialEq)]
pub struct PageSettings {
    /// UI language code: KR, JP, EN, FR, DE or CN.
    pub language_code: String,
    /// "Combine Pets with Owner".
    pub merge_pets: bool,
    pub tooltips: bool,
    /// "Resizing Arrow": the handle in the corner of the window.
    pub corner_handle: bool,
    /// The page's font size as a CSS value (`resolution`): every `rem` in the overlay follows it.
    font_size: String,
    /// The background image's URL; empty for none.
    background_image: String,
    background_size: String,
    background_repeat: String,
    accent: Hex,
}

impl PageSettings {
    /// The only place that knows the page's settings keys.
    pub fn from_raw(raw: &SettingsFile) -> Self {
        let language = raw.text("Lang");
        PageSettings {
            language_code: if language.is_empty() { "EN".to_string() } else { language },
            merge_pets: raw.flag("pets"),
            tooltips: raw.flag("tooltips"),
            corner_handle: raw.flag("arrow"),
            font_size: raw.text("resolution"),
            background_image: if raw.flag("overlayBg") { raw.text("overlayBgImg") } else { String::new() },
            background_size: raw.text("overlayBgSize"),
            background_repeat: raw.text("overlayBgRepeat"),
            accent: raw.color("accent"),
        }
    }

    /// The rule that sets the page-wide variables on `:root`, which in an HTML document is the `<html>` element: there is no element of
    /// the app's own to carry them.
    pub fn root_rule(&self) -> String {
        let image = if self.background_image.is_empty() { "none".to_string() } else { format!("url('{}')", self.background_image.replace('\'', "%27")) };
        let mut vars = Vars::default();
        vars.set("html-font-size", &self.font_size)
            .set("html-bg-image", image)
            .set("html-bg-size", &self.background_size)
            .set("html-bg-repeat", &self.background_repeat)
            .set("accent", format!("#{}", self.accent.as_str()));
        format!(":root{{{}}}", vars.finish())
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/areas/page.rs"]
mod tests;
