//! The coloured bar behind a row and the small pet / overheal / shield bars.
//!
//! Built the way the original builds them, on purpose: the main bar is `position:absolute` with a
//! percentage `width`, and the small bars are `float`s inside a `.mini` container that is as wide
//! as the main bar. That makes the browser snap every edge to whole pixels exactly like the
//! original does, whatever the window width or `resolution` setting.
//!
//! **Animation.** A `width` transition (what the original's jQuery does) cannot run on the compositor
//! thread: every frame the main thread has to lay the page out again, and Chrome reports each such
//! animation as "not composited". So the bars do not animate `width`. A bar's `width` is set to its final
//! value at once, which is what makes the picture at rest identical to the original's, and the change is
//! played as a `transform` (`translateX` + `scaleX`) that starts at the bar's *previous* position and size
//! and ends at no transform. Both the position and the width move linearly with the same easing, as they
//! did before; only the thread doing it changed. See `flip`, and the `chrome-bar-flip-*` classes in
//! `tailwind.css`. (An earlier version sized the bars with `scaleX` permanently, which anti-aliased them
//! on fractional pixels and showed a one-pixel seam at most window widths. Here the transform exists only
//! while a bar is moving, so nothing is left composited at rest.)
//!
//! Only what genuinely differs per row is inline: the widths, the colour and the animation's starting
//! transform. Size, top margin, opacity and corners are CSS variables set once per table
//! (`standard_table.rs`'s `body_style`) and read by the `chrome-bar-*` classes in `tailwind.css`.

use super::table_environment::TableEnvironment;
use crate::infrastructure::act::data::CombatantRecord;
use crate::presentation::ui::shared::palette::{bar_color, percent_of_whole, player_bar_color, with_optional_gradient};
use crate::presentation::ui::shared::style_values::StyleValues;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The graph-bar corner radius (`rd_graph*` x `sizeRadiusGraph`) as a `border-radius` value.
pub(super) fn graph_bar_radius(values: &StyleValues) -> String {
    let [top_left, top_right, bottom_left, bottom_right] = values.corner_radius("rd_graph", "sizeRadiusGraph");
    format!("{top_left} {top_right} {bottom_right} {bottom_left}")
}

/// What is the same for every bar of one table, worked out once instead of once per bar.
pub(super) struct BarSettings {
    animate: bool,
    /// The small bars float against the right edge instead of the left (the HPS and DPS tables
    /// each have their own setting).
    float_side: &'static str,
}

impl BarSettings {
    pub(super) fn new(environment: &TableEnvironment, _values: &StyleValues, is_healing: bool) -> Self {
        let position_key = if is_healing { "bar_position" } else { "bar_position_DPS" };
        BarSettings { animate: environment.animate_bars, float_side: if environment.settings.option_text(position_key) == "right" { "right" } else { "left" } }
    }
}

/// Bar widths in whole percent, truncated like the original's `parseInt`.
struct BarWidths {
    main: i32,
    pet: i32,
    overheal: i32,
    shield: i32,
}

fn bar_widths(combatant: &CombatantRecord, top_value: f64, is_healing: bool) -> BarWidths {
    let (total, contributed_by_pets) =
        if is_healing { (combatant.healed, combatant.pet_effective_healed) } else { (combatant.damage, combatant.pet_damage) };
    BarWidths {
        // The main bar is a share of the table's best value. The pet bar is *also* sized against
        // the table's best value (the original's `inputGraph` divides by `maxDamage`, not by the
        // row's own total) and is then a percentage of the main bar's length -- so it equals "the
        // pet's share of this row" only for the top row. Overheal and shield are shares of the
        // row's own healing.
        main: percent_of_whole(total, top_value).clamp(0, 100),
        pet: percent_of_whole(contributed_by_pets, top_value).clamp(0, 100),
        overheal: percent_of_whole(combatant.over_heal, combatant.healed).clamp(0, 100),
        shield: percent_of_whole(combatant.damage_shield, combatant.healed).clamp(0, 100),
    }
}

/// The `chrome-bar-*` class holding a small bar kind's height, margin, opacity and corners.
fn chrome_class(kind: &str) -> &'static str {
    match kind {
        "oh" => "chrome-bar-oh",
        "ds" => "chrome-bar-ds",
        _ => "chrome-bar-pet",
    }
}

/// Where a bar is, in percent of the row's width: the edge it is anchored to (its left edge, or its right edge for a
/// bar floated right, measured from the row's left) and its width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BarGeometry {
    pub edge: f64,
    pub width: f64,
}

/// The transform that shows a bar at its previous geometry: `translateX(dx% of its own width) scaleX(sx)`, taken
/// about the anchored edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Flip {
    pub dx: f64,
    pub sx: f64,
}

/// The starting transform for a bar that moves from `old` to `new`; `None` when nothing needs animating (it did not
/// move, or it ends up with no width, which cannot be expressed as a scale and just snaps).
pub(super) fn flip(old: Option<BarGeometry>, new: BarGeometry) -> Option<Flip> {
    if new.width <= 0.0 {
        return None;
    }
    let Some(old) = old.filter(|old| old.width > 0.0) else {
        return Some(Flip { dx: 0.0, sx: 0.0 }); // a new bar grows out of its anchored edge
    };
    let flip = Flip { dx: (old.edge - new.edge) / new.width * 100.0, sx: old.width / new.width };
    let unchanged = (flip.sx - 1.0).abs() < 1e-9 && flip.dx.abs() < 1e-9;
    (!unchanged).then_some(flip)
}

/// Geometry of the small bars of one row. Each floats to one side of the `.mini` container, which is as wide as the
/// main bar (`main`, in percent of the row); a bar's own width is a percentage of that container. `None` when they
/// add up to more than the container: the browser then wraps the last one onto a second line, and that is not
/// something a horizontal transform can describe, so those bars just snap.
pub(super) fn small_bar_geometry(main: f64, widths: &[i32], on_right: bool) -> Option<Vec<BarGeometry>> {
    if widths.iter().sum::<i32>() > 100 {
        return None;
    }
    let mut before = 0.0; // the widths of the small bars already placed, in percent of the container
    let mut geometry = Vec::with_capacity(widths.len());
    for &width in widths {
        let edge_in_container = if on_right { 100.0 - before } else { before };
        geometry.push(BarGeometry { edge: edge_in_container * main / 100.0, width: f64::from(width) * main / 100.0 });
        before += f64::from(width);
    }
    Some(geometry)
}

/// How long a bar's remembered position is trusted. After a longer gap (the row was gone, the fight ended) a bar that
/// comes back grows in again instead of sliding from where it used to be.
const REMEMBER_FOR_MS: f64 = 3000.0;

#[derive(Clone, Copy)]
struct Remembered {
    geometry: BarGeometry,
    /// Which of the two identical keyframe sets the bar plays. Switching the animation's name is what restarts a CSS
    /// animation, so every move uses the other one.
    second_keyframes: bool,
    flip: Option<Flip>,
    seen_at_ms: f64,
}

/// What every bar drawn so far looked like, so the next draw knows where each one comes from. Provided once, by the root
/// component. (Plain shared memory rather than a signal: drawing a bar must not cause another draw.)
#[derive(Clone, Default)]
pub struct BarHistory(Rc<RefCell<HashMap<String, Remembered>>>);

/// The classes and inline custom properties that play a bar's move.
#[derive(Default)]
struct BarAnimation {
    keyframes_class: &'static str,
    custom_properties: String,
}

/// Decides the animation of the bar `key` (which row, which bar) now that it should be at `geometry`. Drawing the same
/// bar again without it having moved gives the same answer as before, so nothing changes in the page and a running
/// animation is left alone.
fn animation_of(key: &str, geometry: Option<BarGeometry>, on_right: bool) -> BarAnimation {
    let (Some(history), Some(geometry)) = (try_consume_context::<BarHistory>(), geometry) else {
        return BarAnimation::default();
    };
    let now = js_sys::Date::now();
    let mut remembered = history.0.borrow_mut();
    let previous = remembered.get(key).copied().filter(|previous| now - previous.seen_at_ms < REMEMBER_FOR_MS);
    let current = match previous {
        Some(previous) if previous.geometry == geometry => Remembered { seen_at_ms: now, ..previous },
        Some(previous) => Remembered { geometry, second_keyframes: !previous.second_keyframes, flip: flip(Some(previous.geometry), geometry), seen_at_ms: now },
        None => Remembered { geometry, second_keyframes: false, flip: flip(None, geometry), seen_at_ms: now },
    };
    remembered.insert(key.to_string(), current);
    let Some(flip) = current.flip else { return BarAnimation::default() };
    BarAnimation {
        keyframes_class: if current.second_keyframes { "chrome-bar-flip-b" } else { "chrome-bar-flip-a" },
        custom_properties: format!("--bar-dx:{:.4}%;--bar-sx:{:.6};{}", flip.dx, flip.sx, if on_right { "--bar-origin:right;" } else { "" }),
    }
}

pub(super) fn graph_bars(
    environment: &TableEnvironment,
    combatant: &CombatantRecord,
    top_value: f64,
    is_healing: bool,
    row_id: &str,
    bars: &BarSettings,
) -> Element {
    let settings = environment.settings;
    let widths = bar_widths(combatant, top_value, is_healing);
    let animated = bars.animate;

    let main_background = with_optional_gradient(settings, &player_bar_color(settings, combatant, row_id));

    // The small bars, in the original's DOM order (each floats to the configured side, so the
    // first one sits at the edge). A row whose small bars add up to more than the container wraps
    // its last one onto a second line, exactly as it does in the original: that is the browser's
    // float layout, not something computed here.
    let small_bars: &[(&str, i32, &str)] = if is_healing {
        &[("oh", widths.overheal, "bar_oh"), ("ds", widths.shield, "bar_ds"), ("pet", widths.pet, "bar_pet")]
    } else {
        &[("pet", widths.pet, "bar_pet")]
    };
    let shown: Vec<(&str, i32, &str)> = small_bars.iter().copied().filter(|&(_, _, option_key)| settings.option_enabled(option_key)).collect();
    let on_right = bars.float_side == "right";
    let shown_widths: Vec<i32> = shown.iter().map(|&(_, width, _)| width).collect();
    let small_geometry = small_bar_geometry(f64::from(widths.main), &shown_widths, on_right);
    let mut small_elements = Vec::new();
    for (index, &(kind, width, _)) in shown.iter().enumerate() {
        let background = with_optional_gradient(settings, &bar_color(settings, kind, "", row_id));
        let chrome = chrome_class(kind);
        let motion = if animated { animation_of(&format!("{is_healing}|{row_id}|{kind}"), small_geometry.as_ref().map(|g| g[index]), on_right) } else { BarAnimation::default() };
        small_elements.push(rsx! {
            div {
                key: "{kind}",
                class: "{kind}",
                class: "{chrome}",
                class: if animated { "chrome-bar-animated" },
                class: "{motion.keyframes_class}",
                style: "float:{bars.float_side};width:{width}%;background:{background};{motion.custom_properties}",
            }
        });
    }

    let main_motion = if animated {
        animation_of(&format!("{is_healing}|{row_id}|main"), Some(BarGeometry { edge: 0.0, width: f64::from(widths.main) }), false)
    } else {
        BarAnimation::default()
    };

    rsx! {
        div {
            class: "bar chrome-bar-main",
            class: if animated { "chrome-bar-animated" },
            class: "{main_motion.keyframes_class}",
            style: "width:{widths.main}%;background:{main_background};{main_motion.custom_properties}",
        }
        div { class: "mini", width: "{widths.main}%", {small_elements.into_iter()} }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/combat_tables/graph_bars.rs"]
mod tests;
