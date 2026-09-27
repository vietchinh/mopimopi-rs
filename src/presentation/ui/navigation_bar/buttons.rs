//! Buttons at the right of the top bar. Capture, History and End-encounter appear when pinned
//! in the settings or while the mouse is over the ⋮ button.

use super::capture_screenshot;
use crate::application::app_state::*;
use crate::domain::translations::{translate, translations};
use dioxus::prelude::*;
use crate::presentation::ui::overlay_plugin_context::OverlayPluginContext;

#[component]
pub(super) fn NavigationButtons(is_settings_preview: bool) -> Element {
    let overlay_plugin_context = use_context::<OverlayPluginContext>();
    let context = use_context::<AppContext>();
    let settings = context.settings.read();
    let mouse_is_over_menu_button = *context.nav_buttons_expanded.read();
    let is_encounter_active = overlay_plugin_context.get_is_encounter_active();
    let is_shown = |button: &str| {
        let is_pinned = settings.option_enabled(&format!("btn_{button}"));
        let history_is_available = !(button == "History" && is_encounter_active);
        is_pinned || (mouse_is_over_menu_button && history_is_available)
    };
    let (class, style) = if is_settings_preview { ("right btn_wrap", "top:0") } else { ("right top btn_wrap", "") };

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            onmouseleave: move |_| {
                let mut expanded = context.nav_buttons_expanded;
                expanded.set(false);
                let mut flashing = context.capture_flash_active;
                flashing.set(false);
            },
            if is_shown("Capture") { NavigationButton { name: "Capture", icon: "camera", is_settings_preview } }
            if is_shown("History") { NavigationButton { name: "History", icon: "history", is_settings_preview } }
            // "RequestEnd" (End encounter) is not shown: this backend has no command to end an encounter
            // (OverlayPlugin's WebSocket API documents no such call; the original used the removed legacy
            // ACTWebSocket protocol for it). Automatic ending via OverlayPlugin's own settings still works.
            NavigationButton { name: "More", icon: "more_vert", is_settings_preview }
        }
    }
}

#[component]
fn NavigationButton(name: &'static str, icon: &'static str, is_settings_preview: bool) -> Element {
    let context = use_context::<AppContext>();
    let is_blinking = *context.capture_flash_active.read() && name == "Capture";
    rsx! {
        div {
            "name": name,
            class: "btn flex",
            onmouseenter: move |_| {
                if is_settings_preview { return; }
                show_button_tooltip(context, name);
                if name == "More" {
                    let mut expanded = context.nav_buttons_expanded;
                    expanded.set(true);
                }
            },
            onmouseleave: move |_| {
                let mut tooltip = context.tooltip_html;
                tooltip.set(None);
            },
            onclick: move |_| if !is_settings_preview { press_button(context, name) },
            i { class: if is_blinking { "material-icons flash animated" } else { "material-icons" }, "{icon}" }
        }
    }
}

fn press_button(context: AppContext, name: &str) {
    match name {
        "Capture" => capture_screenshot(context),
        "History" => open_history_screen(context),
        _ => {
            let mut dropdown = context.open_dropdown;
            dropdown.set(Some(Dropdown::Navigation));
        }
    }
}

fn show_button_tooltip(context: AppContext, button_name: &str) {
    let (tooltips_enabled, language) = {
        let settings = context.settings.peek();
        (settings.option_enabled("tooltips"), settings.language_code())
    };
    if tooltips_enabled {
        let text = translate(&translations().ui_schema["NAV"]["main"]["btn"][button_name]["m"], &language);
        let mut tooltip = context.tooltip_html;
        tooltip.set(Some(text));
    }
}


