//! Colours of the graph bars, following the "Palette" setting (job, role, or me / others).

use crate::domain::combat::Player;
use crate::domain::settings::Settings;

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

/// Colour of a player's main bar.
pub fn player_bar_color(settings: &Settings, player: &Player, row_id: &str) -> String {
    bar_color(settings, &player.class_code, player.role.palette_key(), row_id)
}

/// Adds the fade-in gradient when the "gradient" option is on.
pub fn with_optional_gradient(settings: &Settings, color: &str) -> String {
    if !settings.option_enabled("gradient") {
        return color.to_string();
    }
    let direction = match settings.option_text("direction").as_str() {
        direction @ ("top" | "bottom" | "left" | "right") => direction.to_string(),
        _ => "right".to_string(),
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
