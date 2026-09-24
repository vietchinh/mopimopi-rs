//! Pop-up menus. Which one is open is stored in `AppContext::open_dropdown` (see `Dropdown`).
//!
//! * `menu_item`         – one clickable line, optionally with a switch
//! * `navigation_menu`   – the ⋮ menu
//! * `choice_menus`      – pick one value, toggle several options, choose column alignment

mod choice_menus;
mod menu_item;
mod navigation_menu;

use crate::application::app_state::{AppContext, Dropdown};
use dioxus::prelude::*;

#[component]
pub fn DropdownMenu() -> Element {
    let context = use_context::<AppContext>();
    let Some(open_menu) = context.open_dropdown.read().clone() else { return rsx! {} };

    let items = match open_menu {
        Dropdown::Navigation => navigation_menu::navigation_menu_items(context),
        Dropdown::ChooseOption { setting_key, choices } => choice_menus::option_choice_items(context, &setting_key, &choices),
        Dropdown::ToggleOptions { options } => choice_menus::option_toggle_items(context, options),
        Dropdown::ChooseColumnAlignment { column, field } => choice_menus::column_alignment_items(context, &column, &field),
    };
    rsx! { div { class: "dropdown", ul { {items} } } }
}
