//! The typed readers on `SettingsFile`: one per convention of the file. Each takes the key used in `defaults.json` / `l.json`, and in debug
//! builds fails loudly on a key the file does not have, where the string-based reader quietly gives 0, "" or black for a typo.

use super::shapes::{Corners, Line};
use super::values::{FontStack, Opacity, Paint, Shadow, Size};
use crate::domain::settings::{Hex, OptionValue, SettingsFile};

/// A choice in the file read as a type of its own: a number (`applyScope: 3`) or a name (`borderTextType: "outline"`).
pub trait FromSetting {
    fn from_setting(value: &OptionValue) -> Self;
}

impl FromSetting for String {
    fn from_setting(value: &OptionValue) -> Self {
        value.as_text()
    }
}

impl FromSetting for i32 {
    fn from_setting(value: &OptionValue) -> Self {
        value.as_number() as i32
    }
}

pub trait StyleReads {
    /// A size slider (`Range`, tenths of a rem).
    fn size(&self, key: &str) -> Size;
    /// An opacity slider (`Range`, percent).
    fn opacity(&self, key: &str) -> Opacity;
    /// A switch (`q`), with JavaScript truthiness.
    fn flag(&self, key: &str) -> bool;
    /// An option as a number.
    fn number(&self, key: &str) -> f64;
    /// An option as text.
    fn text(&self, key: &str) -> String;
    /// A choice, as the type that knows its options.
    fn choice<T: FromSetting>(&self, key: &str) -> T;
    /// A colour joined with an opacity (`rgba(...)`). Usually both have one key (`Color.tableHd` and `Range.tableHd`).
    fn paint(&self, key: &str) -> Paint;
    /// Some are not: the nav icon is `Color.accent` with `Range.navIcon`.
    fn paint_with(&self, color_key: &str, opacity_key: &str) -> Paint;
    /// A colour on its own (`#hex`), for the texts whose opacity is set separately.
    fn solid(&self, color_key: &str) -> Paint;
    /// The hex digits of a colour.
    fn color(&self, color_key: &str) -> Hex;
    /// The user's font (`q`) and the fallbacks after it.
    fn font(&self, key: &str, fallbacks: &str) -> FontStack;
    /// `{prefix}TL`, `{prefix}TR`, `{prefix}BL`, `{prefix}BR` (a switch each) times the radius slider `radius_key`.
    fn corners(&self, prefix: &str, radius_key: &str) -> Corners;
    /// The text border in the colour `color_key`, a glow or an outline as `borderTextType` says.
    fn shadow(&self, color_key: &str) -> Shadow;
    /// A line of the width slider `width_key`, `solid`, in the colour and opacity of `paint_key`.
    fn solid_line(&self, width_key: &str, paint_key: &str) -> Line;
    /// A line whose style is a choice of the file (`edgeType`).
    fn styled_line(&self, width_key: &str, style_key: &str, paint_key: &str) -> Line;
}

impl SettingsFile {
    fn range(&self, key: &str) -> f64 {
        debug_assert!(self.ranges.contains_key(key), "unknown slider setting `{key}`");
        self.get_range(key).unwrap_or(0.0)
    }

    fn option(&self, key: &str) -> Option<&OptionValue> {
        debug_assert!(self.options.contains_key(key), "unknown option `{key}`");
        self.get_option(key)
    }

    fn hex(&self, key: &str) -> Hex {
        debug_assert!(self.colors.contains_key(key), "unknown colour setting `{key}`");
        Hex::new(self.get_color(key).unwrap_or("000000"))
    }
}

impl StyleReads for SettingsFile {
    fn size(&self, key: &str) -> Size {
        Size(self.range(key))
    }

    fn opacity(&self, key: &str) -> Opacity {
        Opacity(self.range(key))
    }

    fn flag(&self, key: &str) -> bool {
        self.option(key).is_some_and(OptionValue::is_truthy)
    }

    fn number(&self, key: &str) -> f64 {
        self.option(key).map_or(0.0, OptionValue::as_number)
    }

    fn text(&self, key: &str) -> String {
        self.option(key).map(OptionValue::as_text).unwrap_or_default()
    }

    fn choice<T: FromSetting>(&self, key: &str) -> T {
        T::from_setting(self.option(key).unwrap_or(&OptionValue::Text(String::new())))
    }

    fn paint(&self, key: &str) -> Paint {
        self.paint_with(key, key)
    }

    fn paint_with(&self, color_key: &str, opacity_key: &str) -> Paint {
        Paint::with_opacity(self.hex(color_key), self.opacity(opacity_key))
    }

    fn solid(&self, color_key: &str) -> Paint {
        Paint::solid(self.hex(color_key))
    }

    fn color(&self, color_key: &str) -> Hex {
        self.hex(color_key)
    }

    fn font(&self, key: &str, fallbacks: &str) -> FontStack {
        FontStack::new(&self.text(key), fallbacks)
    }

    fn corners(&self, prefix: &str, radius_key: &str) -> Corners {
        let radius = self.range(radius_key);
        let corner = |suffix: &str| Size(self.number(&format!("{prefix}{suffix}")) * radius);
        Corners { top_left: corner("TL"), top_right: corner("TR"), bottom_left: corner("BL"), bottom_right: corner("BR") }
    }

    fn shadow(&self, color_key: &str) -> Shadow {
        let color = self.hex(color_key);
        if self.text("borderTextType") == "outline" { Shadow::Outline(color) } else { Shadow::Glow(color) }
    }

    fn solid_line(&self, width_key: &str, paint_key: &str) -> Line {
        Line { width: self.size(width_key), style: "solid".to_string(), paint: self.paint(paint_key) }
    }

    fn styled_line(&self, width_key: &str, style_key: &str, paint_key: &str) -> Line {
        Line { width: self.size(width_key), style: self.text(style_key), paint: self.paint(paint_key) }
    }
}
