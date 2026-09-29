//! Colours of the graph bars, following the "Palette" setting (job, role, or me / others).

use crate::domain::settings::Settings;
use crate::infrastructure::act::data::CombatantRecord;

const PET_BAR: &str = "pet";
const OVERHEAL_BAR: &str = "oh";
const SHIELD_BAR: &str = "ds";
const LOCAL_PLAYER_ID: &str = "YOU";

/// CSS colour (`#RRGGBB`) of a bar. `color_key` is a job or bar kind ("pet", "ds", "oh");
/// `role_key` the role palette key, empty for the small bars; `row_id` the row's element id.
pub fn bar_color(settings: &Settings, color_key: &str, role_key: &str, row_id: &str) -> String {
    let color = |key: &str| format!("#{}", settings.color_hex(key));
    let is_small_bar = matches!(color_key, PET_BAR | OVERHEAL_BAR | SHIELD_BAR);
    let uses_my_color = settings.option_enabled("myColorUse") && row_id.contains(LOCAL_PLAYER_ID) && !is_small_bar;
    match settings.option_text("palette").as_str() {
        "role" => {
            if uses_my_color {
                color("myColor")
            } else if !role_key.is_empty() && color_key != "LMB" {
                color(role_key)
            } else {
                color(color_key)
            }
        }
        "meYou" => {
            if matches!(color_key, PET_BAR | OVERHEAL_BAR | SHIELD_BAR | "LMB" | "CBO") {
                color(color_key)
            } else if row_id == LOCAL_PLAYER_ID {
                color(LOCAL_PLAYER_ID)
            } else {
                color("Other")
            }
        }
        _ => {
            if uses_my_color { color("myColor") } else { color(color_key) }
        }
    }
}

/// Colour of a player's main bar. The colour key is the row's `class_code()` (not the raw job text the
/// icon uses); `role_palette_key()` is only used in the "role" palette mode.
pub fn player_bar_color(settings: &Settings, player: &CombatantRecord, row_id: &str) -> String {
    bar_color(settings, &player.class_code(), player.role_palette_key(), row_id)
}

/// Adds the fade-in gradient when the "gradient" option is on.
///
/// The original writes `-webkit-linear-gradient(<direction>, transparent, colour)`. In that legacy
/// syntax the keyword names where the gradient *starts* (`top` = start at the top, end at the
/// bottom), the opposite of the standard `linear-gradient(to top, ..)` which names where it *ends*.
/// The setting's labels say "to Top", but what the original actually draws is the legacy meaning,
/// so each direction is mirrored here to draw the same picture.
pub fn with_optional_gradient(settings: &Settings, color: &str) -> String {
    if !settings.option_enabled("gradient") {
        return color.to_string();
    }
    let direction = match settings.option_text("direction").as_str() {
        "top" => "bottom",
        "bottom" => "top",
        "left" => "right",
        "right" => "left",
        // an unknown value makes the whole legacy declaration invalid, so nothing is drawn
        _ => return color.to_string(),
    };
    format!("linear-gradient(to {direction}, rgba(0,0,0,0), {color})")
}

/// `part` as a whole percentage of `whole`, limited to 0-100.
pub fn percent_of_whole(part: f64, whole: f64) -> i32 {
    if whole <= 0.0 {
        return 0;
    }
    let percent = (part / whole * 100.0).trunc();
    if percent.is_finite() { percent.clamp(0.0, 100.0) as i32 } else { 0 }
}
