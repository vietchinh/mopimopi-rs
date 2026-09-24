//! Buttons at the right of the top bar. Capture, History and End-encounter appear when pinned
//! in the settings or while the mouse is over the ⋮ button.

use crate::application::app_state::*;
use crate::infrastructure::network;
use crate::domain::translations::{translate, translations};
use dioxus::prelude::*;
use gloo_timers::callback::Timeout;

const CAPTURE_ICON_BLINK_MILLISECONDS: u32 = 750;
/// ACT needs a moment after the blink before the screenshot request is sent.
const CAPTURE_REQUEST_DELAY_MILLISECONDS: u32 = 1_300;

#[component]
pub(super) fn NavigationButtons(is_settings_preview: bool) -> Element {
    let context = use_context::<AppContext>();
    let settings = context.settings.read();
    let mouse_is_over_menu_button = *context.nav_buttons_expanded.read();
    let fight_is_running = context.latest_combat_data.read().as_ref().map(|data| data.is_encounter_active).unwrap_or(false);
    let is_shown = |button: &str| {
        let is_pinned = settings.option_enabled(&format!("btn_{button}"));
        let history_is_available = !(button == "History" && fight_is_running);
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
            if is_shown("RequestEnd") { NavigationButton { name: "RequestEnd", icon: "timer_off", is_settings_preview } }
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
        "RequestEnd" => network::send_overlay_request("RequestEnd"),
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

/// Screenshot request: blink the icon, ask ACT to save a capture, then show a toast.
pub fn capture_screenshot(context: AppContext) {
    let mut flashing = context.capture_flash_active;
    flashing.set(true);
    Timeout::new(CAPTURE_ICON_BLINK_MILLISECONDS, move || {
        let mut flashing = context.capture_flash_active;
        flashing.set(false);
    })
    .forget();
    Timeout::new(CAPTURE_REQUEST_DELAY_MILLISECONDS, || network::send_overlay_request("Capture")).forget();
    show_toast_message(context, "Capture", 1500, 8000);
}
