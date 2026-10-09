//! The graph bars behind the rows: their colours, the fade, the side the small bars float to, and which small bars show.

use crate::domain::settings::SettingsFile;
use crate::infrastructure::act::data::CombatantRecord;
use crate::presentation::ui::shared::style::StyleReads;
use indexmap::IndexMap;

const PET_BAR: &str = "pet";
const OVERHEAL_BAR: &str = "oh";
const SHIELD_BAR: &str = "ds";
const LOCAL_PLAYER_ID: &str = "YOU";

/// The side the small bars (pet, shield, overheal) float to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    /// The class that floats a bar to this side.
    pub fn float_class(self) -> &'static str {
        match self {
            Side::Left => "float-left",
            Side::Right => "float-right",
        }
    }
}

/// "Palette": the bars follow the player's job, the player's role, or are only told apart as me and the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaletteMode {
    Job,
    Role,
    MeAndOthers,
}

/// The colours of the bars: a colour per job, per role, per small bar, and for me and the others, with the rules that pick one.
#[derive(Clone, Debug, PartialEq)]
pub struct BarPalette {
    mode: PaletteMode,
    /// "Use my own colour": the local player's bar has its own colour.
    uses_my_color: bool,
    /// Every colour of the file, by key (`WAR`, `Tanker`, `pet`, `myColor`, ...): the key of a bar is only known when the bar is drawn.
    colors: IndexMap<String, String>,
}

impl BarPalette {
    /// `#RRGGBB`; black for a key the file does not have.
    fn color(&self, key: &str) -> String {
        format!("#{}", self.colors.get(key).map_or("000000", String::as_str))
    }

    /// CSS colour of a bar. `color_key` is a job or bar kind (`pet`, `ds`, `oh`); `role_key` the role's palette key, empty for the small
    /// bars; `row_id` the row's element id.
    pub fn color_of(&self, color_key: &str, role_key: &str, row_id: &str) -> String {
        let is_small_bar = matches!(color_key, PET_BAR | OVERHEAL_BAR | SHIELD_BAR);
        let uses_my_color = self.uses_my_color && row_id.contains(LOCAL_PLAYER_ID) && !is_small_bar;
        match self.mode {
            PaletteMode::Role => {
                if uses_my_color {
                    self.color("myColor")
                } else if !role_key.is_empty() && color_key != "LMB" {
                    self.color(role_key)
                } else {
                    self.color(color_key)
                }
            }
            PaletteMode::MeAndOthers => {
                if matches!(color_key, PET_BAR | OVERHEAL_BAR | SHIELD_BAR | "LMB" | "CBO") {
                    self.color(color_key)
                } else if row_id == LOCAL_PLAYER_ID {
                    self.color(LOCAL_PLAYER_ID)
                } else {
                    self.color("Other")
                }
            }
            PaletteMode::Job => {
                if uses_my_color { self.color("myColor") } else { self.color(color_key) }
            }
        }
    }

    /// Colour of a player's main bar. The colour key is the row's `class_code()` (not the raw job text the icon uses); the role key is
    /// only used by the role palette.
    pub fn player_color(&self, player: &CombatantRecord, row_id: &str) -> String {
        self.color_of(&player.class_code(), player.role_palette_key(), row_id)
    }
}

/// The fade-in gradient of the bars ("Gradient" and its direction).
///
/// The original writes `-webkit-linear-gradient(<direction>, transparent, colour)`. In that legacy syntax the keyword names where the
/// gradient *starts* (`top` = start at the top, end at the bottom), the opposite of the standard `linear-gradient(to top, ..)` which names
/// where it *ends*. The setting's labels say "to Top", but what the original actually draws is the legacy meaning, so each direction is
/// mirrored here to draw the same picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fade {
    None,
    /// The side the standard gradient goes `to`.
    Toward(&'static str),
}

impl Fade {
    /// The background of a bar of this colour.
    pub fn apply(self, color: &str) -> String {
        match self {
            Fade::None => color.to_string(),
            Fade::Toward(direction) => format!("linear-gradient(to {direction}, rgba(0,0,0,0), {color})"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BarSettings {
    /// "Animation": the bars move to their new width.
    pub animate: bool,
    damage_side: Side,
    healing_side: Side,
    pub pet: bool,
    pub shield: bool,
    pub overheal: bool,
    pub palette: BarPalette,
    pub fade: Fade,
}

impl BarSettings {
    /// The only place that knows the bars' settings keys.
    pub fn from_raw(raw: &SettingsFile) -> Self {
        let side = |key: &str| if raw.text(key) == "right" { Side::Right } else { Side::Left };
        let fade = if raw.flag("gradient") {
            match raw.text("direction").as_str() {
                "top" => Fade::Toward("bottom"),
                "bottom" => Fade::Toward("top"),
                "left" => Fade::Toward("right"),
                "right" => Fade::Toward("left"),
                // an unknown value makes the whole legacy declaration invalid, so nothing is drawn
                _ => Fade::None,
            }
        } else {
            Fade::None
        };
        BarSettings {
            animate: raw.flag("ani"),
            damage_side: side("bar_position_DPS"),
            healing_side: side("bar_position"),
            pet: raw.flag("bar_pet"),
            shield: raw.flag("bar_ds"),
            overheal: raw.flag("bar_oh"),
            palette: BarPalette {
                mode: match raw.text("palette").as_str() {
                    "role" => PaletteMode::Role,
                    "meYou" => PaletteMode::MeAndOthers,
                    _ => PaletteMode::Job,
                },
                uses_my_color: raw.flag("myColorUse"),
                colors: raw.colors.iter().map(|(key, hex)| (key.clone(), hex.as_str().to_string())).collect(),
            },
            fade,
        }
    }

    /// The side the small bars of the damage or the healing table float to.
    pub fn side(&self, is_healing: bool) -> Side {
        if is_healing { self.healing_side } else { self.damage_side }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/areas/bars.rs"]
mod tests;
