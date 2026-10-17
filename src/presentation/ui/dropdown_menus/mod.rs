//! Pop-up menus. Which one is open is stored in `DropdownContext::open_dropdown` (see `Dropdown`).
//!
//! * `menu_item`         – one clickable line, optionally with a switch
//! * `navigation_menu`   – the ⋮ menu
//! * `choice_menus`      – pick one value, toggle several options, choose column alignment

mod choice_menus;
mod menu_item;
mod navigation_menu;

use crate::application::app_state::{AppActions, Dropdown, DropdownContext, ScreenContext, SettingsContext, SettingsScreenContext};
use dioxus::prelude::*;

#[component]
pub fn DropdownMenu() -> Element {
    let dropdown = use_context::<DropdownContext>();
    // Hooks first: they must run on every render, including the early return below.
    let settings = use_context::<SettingsContext>();
    let actions = use_context::<AppActions>();
    let screen = use_context::<ScreenContext>();
    let settings_screen = use_context::<SettingsScreenContext>();
    let Some(open_menu) = dropdown.open_dropdown.read().clone() else { return rsx! {} };

    let items = match open_menu {
        Dropdown::Navigation => navigation_menu::navigation_menu_items(actions, screen, settings, settings_screen),
        Dropdown::ChooseOption { setting_key, choices } => choice_menus::option_choice_items(settings, &setting_key, &choices),
        Dropdown::ToggleOptions { options } => choice_menus::option_toggle_items(settings, options),
        Dropdown::ChooseColumnAlignment { column, field } => choice_menus::column_alignment_items(settings, &column, &field),
    };
    rsx! { div { class: "dropdown", ul { {items} } } }
}
