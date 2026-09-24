//! The coloured bar behind a row and the small pet / overheal / shield bars.

use super::table_environment::TableEnvironment;
use crate::domain::combat::{EncounterRanking, Player, TableKind};
use crate::presentation::ui::shared::palette::{bar_color, percent_of_whole, player_bar_color, with_optional_gradient};
use dioxus::prelude::*;

/// Bar widths in whole percent (0-100).
struct BarWidths {
    main_bar: i32,
    pet_bar: i32,
    overheal_bar: i32,
    shield_bar: i32,
}

fn bar_widths(environment: &TableEnvironment, ranking: &EncounterRanking, table: TableKind, player: &Player) -> BarWidths {
    let (total, contributed_by_pets) = match table {
        TableKind::Damage => (player.merged_stats.damage, player.merged_stats.damage - player.own_stats.damage),
        TableKind::Healing => (
            player.merged_stats.healed,
            player.merged_stats.effective_healed - player.own_stats.effective_healed,
        ),
    };
    let pets_are_merged = environment.settings.option_enabled("pets");
    BarWidths {
        main_bar: percent_of_whole(total, ranking.top_value),
        pet_bar: if pets_are_merged { percent_of_whole(contributed_by_pets, ranking.top_value) } else { 0 },
        overheal_bar: percent_of_whole(player.merged_stats.over_heal, player.merged_stats.healed),
        shield_bar: percent_of_whole(player.merged_stats.damage_shield, player.merged_stats.healed),
    }
}

pub(super) fn graph_bars(environment: &TableEnvironment, ranking: &EncounterRanking, table: TableKind, player: &Player, row_id: &str) -> Element {
    let settings = environment.settings;
    let widths = bar_widths(environment, ranking, table, player);
    let transition = if environment.animate_bars { "transition:width .3s;" } else { "" };
    let main_background = with_optional_gradient(settings, &player_bar_color(settings, player, row_id));

    // One of the small bars, drawn only when its option is on.
    let small_bar = |bar_kind: &str, width: i32, option_key: &str| -> Element {
        let background = with_optional_gradient(settings, &bar_color(settings, bar_kind, "", row_id));
        let is_enabled = settings.option_enabled(option_key);
        rsx! { if is_enabled { div { class: "{bar_kind}", style: "width:{width}%;background:{background};{transition}" } } }
    };
    let main_width = widths.main_bar;
    rsx! {
        div { class: "bar", style: "width:{main_width}%;background:{main_background};{transition}" }
        div { class: "mini", style: "width:{main_width}%;{transition}",
            match table {
                TableKind::Damage => rsx! { {small_bar("pet", widths.pet_bar, "bar_pet")} },
                TableKind::Healing => rsx! {
                    {small_bar("oh", widths.overheal_bar, "bar_oh")}
                    {small_bar("ds", widths.shield_bar, "bar_ds")}
                    {small_bar("pet", widths.pet_bar, "bar_pet")}
                },
            }
        }
    }
}
