//! Buttons at the right of the top bar. Capture, History and End-encounter appear when pinned
//! in the settings or while the mouse is over the ⋮ button.

use super::capture_screenshot;
use crate::application::app_state::{AppContext, open_history_screen, Dropdown};
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;
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
    // Non-preview: `nav-btn-wrap-position` (inherits --nav-edge-width from the parent nav) overrides
    // the static `.right{right:0}`/`.top{top:0}` defaults by cascade order once `edge != 0`, matching
    // how the original's dynamically-injected rule always won that same tie. Preview instead always
    // forces exactly `right:0;top:0` regardless of the edge setting (matching the original's separate
    // `.previewArea nav[name=main] .btn_wrap{right:0;top:0}` rule) -- done as explicit inline `right`/`top`
    // rather than relying on class order, since inline styles win regardless of stylesheet load order.
    let class = if is_settings_preview {
        "right btn_wrap nav-btn-wrap-bg"
    } else {
        "right top btn_wrap nav-btn-wrap-bg nav-btn-wrap-position"
    };

    rsx! {
        div {
            class: "{class}",
            right: if is_settings_preview { "0" },
            top: if is_settings_preview { "0" },
            onmouseleave: move |_| {
                let mut expanded = context.nav_buttons_expanded;
                expanded.set(false);
                let mut flashing = context.capture_flash_active;
                flashing.set(false);
            },
            if is_shown("Capture") { NavigationButton { name: "Capture", icon: "camera", is_settings_preview, more_button: false } }
            if is_shown("History") { NavigationButton { name: "History", icon: "history", is_settings_preview, more_button: false } }
            // The original shows this button too (pinned with "Fixed of End-Encounter Button", or while the
            // ⋮ menu is hovered). Pressing it flashes the icon and calls `OverlayPluginApi.endEncounter()`
            // when the host provides one -- which only the in-game overlay does; everywhere else the
            // original's call fails inside a try/catch, exactly as it does here.
            if is_shown("RequestEnd") { NavigationButton { name: "RequestEnd", icon: "timer_off", is_settings_preview, more_button: false } }
            NavigationButton { name: "More", icon: "more_vert", is_settings_preview, more_button: true }
        }
    }
}

#[component]
fn NavigationButton(name: &'static str, icon: &'static str, is_settings_preview: bool, more_button: bool) -> Element {
    let context = use_context::<AppContext>();
    // The original flashes a pressed button's icon once (adds `flash animated`, removes it on animationend).
    let mut flash_once = use_signal(|| false);
    let is_blinking = (*context.capture_flash_active.read() && name == "Capture") || *flash_once.read();
    rsx! {
        div {
            "name": name,
            class: "btn flex",
            class: if more_button { "nav-more-button" },
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
            onclick: move |_| {
                if !is_settings_preview {
                    if name == "RequestEnd" {
                        flash_once.set(true);
                    }
                    press_button(context, name)
                }
            },
            // `MaterialIcon` takes no class, so the classes live on a wrapper that draws no box (`display: contents`, see app.css)
            // and the stylesheet reaches the icon inside it. The animation's end event bubbles up to the wrapper.
            span {
                class: "nav-icon",
                class: if is_blinking { "flash animated" },
                onanimationend: move |_| flash_once.set(false),
                MaterialIcon { name: icon }
            }
        }
    }
}

fn press_button(context: AppContext, name: &str) {
    match name {
        "Capture" => capture_screenshot(context),
        "History" => open_history_screen(context),
        "RequestEnd" => request_end_encounter(),
        _ => {
            let mut dropdown = context.open_dropdown;
            dropdown.set(Some(Dropdown::Navigation));
        }
    }
}

/// `window.OverlayPluginApi.endEncounter()`, if the page is hosted by something that provides it.
/// Any failure is ignored: the original wraps the same call in a try/catch.
fn request_end_encounter() {
    let Some(window) = web_sys::window() else { return };
    let Ok(api) = js_sys::Reflect::get(&window, &"OverlayPluginApi".into()) else { return };
    if api.is_undefined() || api.is_null() {
        return;
    }
    let Ok(function) = js_sys::Reflect::get(&api, &"endEncounter".into()) else { return };
    if let Some(function) = wasm_bindgen::JsCast::dyn_ref::<js_sys::Function>(&function) {
        let _ = function.call0(&api);
    }
}

fn show_button_tooltip(context: AppContext, button_name: &str) {
    let tooltips_enabled = context.settings.peek().option_enabled("tooltips");
    if tooltips_enabled {
        let text = translate(&translations().ui_schema["NAV"]["main"]["btn"][button_name]["m"]);
        let mut tooltip = context.tooltip_html;
        tooltip.set(Some(text));
    }
}


