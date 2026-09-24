//! Root component: builds the shared state and the page shell around the current screen.

use super::dropdown_menus::DropdownMenu;
use super::history_screen::{HistoryNavigationBar, HistoryScreen};
use super::navigation_bar::NavigationBar;
use super::overlays::{Toast, Tooltip};
use super::settings_screens::{SettingsNavigationBar, SettingsScreen};
use super::start_screen::MainScreen;
use crate::models::act_data::CombatDataMessage;
use crate::application::app_state::*;
use crate::domain::combat::build_rankings;
use crate::infrastructure::network::{self, ActEventCallbacks, ConnectionStatus};
use crate::domain::settings::Settings;
use crate::presentation::theme::build_theme_css;
use dioxus::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;

/// The functions the network layer calls when something arrives from ACT.
pub fn make_act_callbacks(context: AppContext) -> ActEventCallbacks {
    ActEventCallbacks {
        on_combat_data: Rc::new(move |message| handle_combat_data_received(context, message)),
        on_local_player_name: Rc::new(move |name| {
            let mut local_player_name = context.local_player_name;
            local_player_name.set(name);
        }),
        on_status_change: Rc::new(move |status: ConnectionStatus| {
            let mut connection_status = context.connection_status;
            connection_status.set(status);
        }),
    }
}

/// Creates every piece of shared state (see `app_state::AppContext` for what each one means).
fn create_app_context() -> AppContext {
    let settings = use_signal(Settings::load_from_browser);
    let displayed_combat_data = use_signal(|| None::<Rc<CombatDataMessage>>);
    let local_player_name = use_signal(String::new);

    // Recomputed only when the data, the pet-merging setting or the player name changes.
    let merge_pets = use_memo(move || settings.read().option_enabled("pets"));
    let rankings = use_memo(move || {
        let message = displayed_combat_data.read().clone()?;
        let name = local_player_name.read().clone();
        Some(Rc::new(build_rankings(&message, merge_pets(), &name)))
    });
    let sample_rankings = use_memo(move || Rc::new(build_rankings(sample_combat_message(), merge_pets(), "")));

    AppContext {
        settings,
        displayed_combat_data,
        local_player_name,
        rankings,
        sample_rankings,
        current_screen: use_signal(|| Screen::Main),
        latest_combat_data: use_signal(|| None::<Rc<CombatDataMessage>>),
        has_received_data: use_signal(|| false),
        encounter_was_active: use_signal(|| false),
        encounter_history: use_signal(Vec::<HistoryEntry>::new),
        encounters_in_current_zone: use_signal(|| 0usize),
        viewed_history_key: use_signal(|| None::<String>),
        connection_status: use_signal(|| ConnectionStatus::Idle),
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
    use_context_provider(|| context);

    // Start listening to ACT once. Deferred so no signal is written while rendering.
    use_hook(|| {
        spawn(async move {
            network::start_listening(make_act_callbacks(context));
        });
    });
    // Save the settings shortly after the last change (see `settings_saving`).
    use_hook(|| register_save_on_page_hide(context.settings));
    use_effect(move || {
        let _ = context.settings.read(); // re-run after every change
        schedule_settings_save(context.settings);
    });

    // Rebuilt only when the settings change, not when the screen or a dropdown does.
    let theme_css = use_memo(move || build_theme_css(&context.settings.read()));
    let screen = *context.current_screen.read();

    rsx! {
        document::Link { rel: "stylesheet", href: "https://fonts.googleapis.com/icon?family=Material+Icons" }
        document::Link { rel: "stylesheet", href: "https://fonts.googleapis.com/css?family=Roboto+Condensed" }
        document::Link { rel: "stylesheet", href: "mopimopi.css" }
        document::Link { rel: "stylesheet", href: "app.css" }
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
