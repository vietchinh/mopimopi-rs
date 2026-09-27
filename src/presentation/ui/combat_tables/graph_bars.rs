//! The coloured bar behind a row and the small pet / overheal / shield bars.

use super::table_environment::TableEnvironment;
use crate::infrastructure::act::data::CombatantRecord;
use crate::presentation::ui::shared::palette::{bar_color, percent_of_whole, player_bar_color, with_optional_gradient};
use dioxus::prelude::*;

/// Bar widths in whole percent (0-100).
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
        // The main bar's width is relative to the table's best value; the small bars are drawn
        // nested inside a div already sized to the main bar's width (see `mini` below), so their
        // own widths must be relative to this row's own total, not to the table's top value.
        main: percent_of_whole(total, top_value),
        pet: percent_of_whole(contributed_by_pets, total),
        overheal: percent_of_whole(combatant.over_heal, combatant.healed),
        shield: percent_of_whole(combatant.damage_shield, combatant.healed),
    }
}

pub(super) fn graph_bars(environment: &TableEnvironment, combatant: &CombatantRecord, top_value: f64, is_healing: bool, row_id: &str) -> Element {
    let settings = environment.settings;
    let widths = bar_widths(combatant, top_value, is_healing);
    let transition = if environment.animate_bars { "transition:width .3s;" } else { "" };
    let main_background = with_optional_gradient(settings, &player_bar_color(settings, combatant, row_id));

    // One of the small bars, drawn only when its option is on.
    let small_bar = |bar_kind: &str, width: i32, option_key: &str| -> Element {
        let background = with_optional_gradient(settings, &bar_color(settings, bar_kind, "", row_id));
        let is_enabled = settings.option_enabled(option_key);
        rsx! { if is_enabled { div { class: "{bar_kind}", style: "width:{width}%;background:{background};{transition}" } } }
    };
    let main_width = widths.main;
    rsx! {
        div { class: "bar", style: "width:{main_width}%;background:{main_background};{transition}" }
        div { class: "mini", style: "width:{main_width}%;{transition}",
            if is_healing {
                {small_bar("oh", widths.overheal, "bar_oh")}
                {small_bar("ds", widths.shield, "bar_ds")}
                {small_bar("pet", widths.pet, "bar_pet")}
            } else {
                {small_bar("pet", widths.pet, "bar_pet")}
            }
        }
    }
}
