//! The navigation bar: its style, its layout, which buttons are pinned and which parts of the summary line show.

use crate::domain::settings::SettingsFile;
use crate::presentation::ui::shared::style::{Corners, FontStack, Line, Opacity, Paint, Shadow, Size, StyleReads, TextStyle, Vars};
use std::fmt::{self, Display};

/// "Display Type of Combatant Data": the summary below the target (2 lines) or beside it.
const TWO_LINE_LAYOUT: i32 = 2;

/// The pattern drawn over the bar's colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pattern {
    Cross,
    HorizontalStripes,
    VerticalStripes,
    DiagonalLeft,
    DiagonalRight,
}

/// The bar's background: its colour, or one of the patterns drawn over it. The gradients keep the original's legacy `-webkit-` syntax,
/// whose first argument is the side the gradient starts at.
#[derive(Clone, Debug, PartialEq)]
pub enum NavBackground {
    Plain(Paint),
    Patterned { pattern: Pattern, color: Paint, base: Paint },
}

impl Display for NavBackground {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NavBackground::Plain(base) => write!(f, "{base}"),
            NavBackground::Patterned { pattern, color, base } => match pattern {
                Pattern::Cross => write!(f, "-webkit-linear-gradient({color},transparent .1rem),-webkit-linear-gradient(0,{color},{base} .1rem)"),
                Pattern::HorizontalStripes => write!(f, "-webkit-linear-gradient({color},transparent .1rem),-webkit-linear-gradient(0,{color},{base} 0)"),
                Pattern::VerticalStripes => write!(f, "-webkit-linear-gradient({color},transparent 0),-webkit-linear-gradient(0,{color},{base} .1rem)"),
                Pattern::DiagonalLeft => write!(f, "repeating-linear-gradient(45deg,{color} 0,{color} 5%,{base} 0,{base} 50%) 0"),
                Pattern::DiagonalRight => write!(f, "repeating-linear-gradient(135deg,{color} 0,{color} 5%,{base} 0,{base} 50%) 0"),
            },
        }
    }
}

/// One of the texts of the bar (the time, the target, the summary).
#[derive(Clone, Debug, PartialEq)]
pub struct NavText {
    /// The colour with its opacity baked in.
    pub color: Paint,
    pub font: FontStack,
    pub size: Size,
    pub italic: bool,
    /// Its opacity slider is at 0: the text gets no size and no padding, instead of relying on a later rule to override them.
    pub hidden: bool,
}

/// The buttons the user pinned to the bar (the ⋮ button is always there).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinnedButtons {
    pub capture: bool,
    pub history: bool,
    pub request_end: bool,
}

/// Which parts of the summary line show ("Display Type of Combatant Data").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SummaryParts {
    pub total_dps: bool,
    pub total_hps: bool,
    pub max_hit_damage: bool,
    pub max_hit_heal: bool,
    pub rank: bool,
    pub max_hit: bool,
    /// The max-hit part shows the strongest heal instead of the strongest hit (the swap button).
    pub shows_strongest_heal: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavSettings {
    pub background: NavBackground,
    pub height: Size,
    pub pattern_size: Size,
    pub corners: Corners,
    pub icon_size: Size,
    pub icon_color: Paint,
    /// `None` when the edge's opacity is 0.
    pub edge: Option<Line>,
    /// The buttons' own background is clear unless the bar is fully opaque.
    pub buttons_transparent: bool,
    pub time: NavText,
    pub target: NavText,
    pub summary_text: NavText,
    pub gap: Size,
    /// The summary is below the target (2 lines) rather than beside it.
    pub two_lines: bool,
    pub pinned: PinnedButtons,
    pub summary: SummaryParts,
}

impl NavSettings {
    /// The only place that knows the nav bar's settings keys.
    pub fn from_raw(raw: &SettingsFile) -> Self {
        let pattern = match raw.text("pattern").as_str() {
            "cross" => Some(Pattern::Cross),
            "hStripe" => Some(Pattern::HorizontalStripes),
            "vStripe" => Some(Pattern::VerticalStripes),
            "leftDig" => Some(Pattern::DiagonalLeft),
            "rightDig" => Some(Pattern::DiagonalRight),
            _ => None,
        };
        let base = raw.paint("navBg");
        let background = match pattern {
            Some(pattern) => NavBackground::Patterned { pattern, color: raw.paint("pattern"), base },
            None => NavBackground::Plain(base),
        };
        NavSettings {
            background,
            height: raw.size("sizeNav"),
            pattern_size: raw.size("sizePattern"),
            corners: raw.corners("rd_nav", "sizeRadius"),
            icon_size: raw.size("sizeIcon"),
            icon_color: raw.paint_with("accent", "navIcon"),
            edge: (!raw.opacity("edge").is_zero()).then(|| raw.styled_line("sizeEdge", "edgeType", "edge")),
            buttons_transparent: raw.opacity("navBg") != crate::presentation::ui::shared::style::Opacity::FULL,
            time: NavText {
                color: raw.paint_with("accent", "navTime"),
                font: raw.font("fTime", "'DS-Digital', 'sans-serif'"),
                size: raw.size("sizeTime"),
                italic: raw.flag("time_italic"),
                hidden: raw.opacity("navTime").is_zero(),
            },
            target: NavText {
                color: raw.paint_with("target", "target"),
                font: raw.font("fTarget", "'Segoe UI', 'sans-serif'"),
                size: raw.size("sizeTarget"),
                italic: raw.flag("target_italic"),
                hidden: raw.opacity("target").is_zero(),
            },
            summary_text: NavText {
                color: raw.paint("rps"),
                font: raw.font("fRPS", "'Roboto Condensed', 'Segoe UI', 'sans-serif'"),
                size: raw.size("sizeRPS"),
                italic: raw.flag("rps_italic"),
                hidden: false,
            },
            gap: raw.size("sizeGap"),
            two_lines: raw.choice::<i32>("act") == TWO_LINE_LAYOUT,
            pinned: PinnedButtons { capture: raw.flag("btn_Capture"), history: raw.flag("btn_History"), request_end: raw.flag("btn_RequestEnd") },
            summary: SummaryParts {
                total_dps: raw.flag("act_rd"),
                total_hps: raw.flag("act_rh"),
                max_hit_damage: raw.flag("act_md"),
                max_hit_heal: raw.flag("act_mh"),
                rank: raw.flag("act_rank"),
                max_hit: raw.flag("act_max"),
                shows_strongest_heal: raw.flag("swap"),
            },
        }
    }

    /// The only place that knows the nav bar's variable names: set once on the `nav` element, inherited by everything in it.
    pub fn bar_vars(&self) -> String {
        let mut vars = Vars::default();
        vars.set("nav-bg", &self.background).set("nav-height", self.height).set("nav-pattern-size", self.pattern_size);
        self.corners.write(&mut vars, "nav-radius");
        vars.set("nav-more-radius-tr", self.corners.top_right).set("nav-more-radius-br", self.corners.bottom_right).set("nav-icon-size", self.icon_size);
        match &self.edge {
            Some(edge) => vars.set("nav-border", edge).set("nav-btn-wrap-bg", self.buttons_background()).set("nav-edge-width", edge.width),
            None => vars.set("nav-border", "unset").set("nav-btn-wrap-bg", self.buttons_background()).set("nav-edge-width", "unset"),
        };
        vars.set("nav-icon-color", &self.icon_color);
        vars.set("nav-time-padding", self.time.padding()).set("nav-target-padding", self.target.padding());
        vars.set("nav-gap", self.gap);
        vars.finish()
    }

    fn buttons_background(&self) -> &'static str {
        if self.buttons_transparent { "transparent" } else { "unset" }
    }

    /// The variables of the time's text, for its cells.
    pub fn time_vars(&self) -> String {
        self.time.text_vars(true)
    }

    /// The variables of the target's text, for its cells.
    pub fn target_vars(&self) -> String {
        self.target.text_vars(true)
    }

    /// The variables of the summary's text, for its cells.
    pub fn summary_vars(&self) -> String {
        self.summary_text.text_vars(false)
    }
}

impl NavText {
    /// `--text-*` for the cells that show this text. A text that can be hidden has no size at all when it is: the space it would take
    /// is the padding's (see `padding`).
    fn text_vars(&self, can_be_hidden: bool) -> String {
        let hidden = can_be_hidden && self.hidden;
        let style = TextStyle {
            color: self.color.clone(),
            // the colour has its opacity in it; nothing else here is dimmed or bordered
            opacity: Opacity::FULL,
            ex_color: None,
            font: self.font.clone(),
            size: Some(if hidden { Size::ZERO } else { self.size }),
            italic: self.italic,
            bold: false,
            shadow: Shadow::None,
        };
        let mut vars = Vars::default();
        style.write(&mut vars, "text");
        vars.finish()
    }

    /// The space before a text that can be hidden: none when it is.
    fn padding(&self) -> &'static str {
        if self.hidden { "0" } else { "1rem" }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/areas/nav.rs"]
mod tests;
