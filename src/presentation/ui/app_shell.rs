//! Root component: builds the shared state and the page shell around the current screen.

use super::dropdown_menus::DropdownMenu;
use super::history_screen::{HistoryNavigationBar, HistoryScreen};
use super::navigation_bar::NavigationBar;
use super::overlay_plugin_context;
use super::overlay_plugin_context::OverlayPluginContext;
use super::overlays::{Toast, Tooltip};
use super::settings_screens::{SettingsNavigationBar, SettingsScreen};
use super::start_screen::MainScreen;
use crate::application::app_state::{AppContext, Screen, SettingsLocation, Dropdown, ToastState, on_combat_data_changed, register_save_on_page_hide, schedule_settings_save, restart_standby_timer};
use crate::domain::settings::Settings;
use crate::presentation::theme::build_theme_css;
use dioxus::prelude::*;
use std::collections::HashSet;

/// Creates every piece of shared state (see `app_state::AppContext` for what each one means).
fn create_app_context() -> AppContext {
    let settings = use_signal(Settings::load_from_browser);

    AppContext {
        settings,
        displayed_combat_data: use_signal(|| None),
        local_player_name: use_signal(String::new),
        current_screen: use_signal(|| Screen::Main),
        has_received_data: use_signal(|| false),
        encounter_was_active: use_signal(|| false),
        encounter_history: use_signal(Vec::new),
        encounters_in_current_zone: use_signal(|| 0usize),
        viewed_history_key: use_signal(|| None::<String>),
        settings_location: use_signal(SettingsLocation::top_level),
        open_dropdown: use_signal(|| None::<Dropdown>),
        settings_preview_enabled: use_signal(|| false),
        settings_preview_raid_mode: use_signal(|| false),
        toast_message: use_signal(ToastState::default),
        toast_generation: use_signal(|| 0u32),
        tooltip_html: use_signal(|| None::<String>),
        is_standby_hidden: use_signal(|| false),
        blurred_player_rows: use_signal(HashSet::<String>::new),
        nav_buttons_expanded: use_signal(|| false),
        capture_flash_active: use_signal(|| false),
    }
}

#[component]
pub fn App() -> Element {
    let context = create_app_context();

    let connection_status = use_signal(|| overlay_plugin_context::ConnectionStatus::NotConfigured);
    let combat_data_message = use_signal(|| None);
    let player_name_signal = use_signal(String::new);
    let connection_error = use_signal(|| None);
    let merge_pets_into_owner = use_signal(|| context.settings.peek().option_enabled("pets"));

    let overlay_plugin_context =
        OverlayPluginContext::new(connection_status, combat_data_message, player_name_signal, connection_error, merge_pets_into_owner);
    overlay_plugin_context::spawn_connection_task(connection_status, combat_data_message, player_name_signal, connection_error, merge_pets_into_owner);

    use_context_provider(|| context);
    use_context_provider(|| overlay_plugin_context);

    use_hook(|| restart_standby_timer(context));

    use_effect(move || {
        let merge_pets = context.settings.read().option_enabled("pets");
        overlay_plugin_context.set_merge_pets_into_owner(merge_pets);
    });

    use_effect(move || {
        if let Some(message) = overlay_plugin_context.combat_data_message() {
            on_combat_data_changed(context, message);
        }
    });
    use_effect(move || {
        let name = overlay_plugin_context.local_player_name();
        if !name.is_empty() {
            let mut local_player_name = context.local_player_name;
            local_player_name.set(name);
        }
    });

    // Save the settings shortly after the last change (see `settings_saving`).
    use_hook(|| register_save_on_page_hide(context.settings));
    use_effect(move || {
        let _ = context.settings.read(); // re-run after every change
        schedule_settings_save(context.settings);
    });

    use_effect(move || {
        let _ = context.current_screen.read();
        let _ = context.settings_location.read();
        let mut tooltip = context.tooltip_html;
        if tooltip.peek().is_some() {
            tooltip.set(None);
        }
    });

    // Rebuilt only when the settings change, not when the screen or a dropdown does.
    let theme_css = use_memo(move || build_theme_css(&context.settings.read()));
    let screen = *context.current_screen.read();

    rsx! {
        style { "{theme_css}" }
        div { id: "wrap",
            if context.open_dropdown.read().is_some() {
                DropdownMenu {}
                div {
                    id: "blackBg",
                    onclick: move |_| {
                        let mut dropdown = context.open_dropdown;
                        dropdown.set(None);
                    },
                }
            }
            match screen {
                Screen::Main => rsx! { NavigationBar { is_settings_preview: false } },
                Screen::History => rsx! { HistoryNavigationBar {} },
                Screen::Settings => rsx! { SettingsNavigationBar {} },
            }
            div { id: "content",
                Tooltip {}
                Toast {}
                match screen {
                    Screen::Main => rsx! { MainScreen {} },
                    Screen::History => rsx! { HistoryScreen {} },
                    Screen::Settings => rsx! { SettingsScreen {} },
                }
            }
        }
    }
}
