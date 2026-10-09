//! The one connection to `OverlayPlugin`, and everything it reports, as Dioxus context.
//!
//! Its signals are created once, in `ActConnection` (see `new` and `spawn_connection_task`),
//! configured only from the
//! page's URL (`?OVERLAY_WS=` / `?HOST_PORT=`). It connects immediately, subscribes, and
//! reconnects on its own with a growing delay. Whether pets are merged into their owners is a
//! setting the GUI owns (see `Settings`, key `"pets"`); the connection task reads it (without subscribing) when a
//! message arrives and hands the parsed message straight to `on_combat_data_changed`.

use crate::infrastructure::act::data::{peek_combat_data, CombatDataMessage, ParseOptions};
use crate::infrastructure::act::overlay_plugin_protocol::{interpret, subscribe_message, OverlayPluginEvent, OverlayPluginUrl, OverlayPluginUrlError};
use crate::infrastructure::browser_websocket::BrowserWebsocketClient;
use crate::application::app_state::{on_combat_data_changed, AppActions, TablesContext};
use crate::presentation::ui::areas::SettingsView;
use dioxus::prelude::*;
use dioxus::signals::{ReadableExt, WritableExt};
use futures::channel::mpsc;
use futures::{select_biased, StreamExt};
use gloo_timers::future::TimeoutFuture;
use std::fmt;
use std::rc::Rc;

/// State of the link to `OverlayPlugin`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// The page's URL has no usable `?OVERLAY_WS=` / `?HOST_PORT=`.
    NotConfigured,
    /// Trying to open the connection. `attempt` counts tries in a row (1 = the first).
    Connecting { attempt: u32 },
    Connected,
    /// Closed or could not be opened; the next attempt starts in `retry_in_seconds`.
    Disconnected { retry_in_seconds: u32 },
}

/// Seconds to wait after `failed_attempts` attempts in a row failed to open: 1, 1, 2, 4, 8, then
/// 15 from then on. A connection that was open and then dropped is retried after 1 second
/// instead (see `keep_connected`), since that is usually ACT restarting, not a wrong address.
fn retry_delay_seconds(failed_attempts: u32) -> u32 {
    match failed_attempts {
        0..=2 => 1,
        3 => 2,
        4 => 4,
        5 => 8,
        _ => 15,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartError(String);

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not connect: {}", self.0)
    }
}

#[derive(Clone, Copy)]
pub struct OverlayPluginContext {
    connection_status: Signal<ConnectionStatus>,
    combat_data_message: Signal<Option<Rc<CombatDataMessage>>>,
    error: Signal<Option<String>>,
}

impl OverlayPluginContext {
    pub fn new(
        connection_status: Signal<ConnectionStatus>,
        combat_data_message: Signal<Option<Rc<CombatDataMessage>>>,
        error: Signal<Option<String>>,
    ) -> OverlayPluginContext {
        OverlayPluginContext { connection_status, combat_data_message, error }
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        *self.connection_status.read()
    }

    pub fn error_message(&self) -> Option<String> {
        self.error.read().clone()
    }

    pub fn get_is_encounter_active(&self) -> bool {
        self.combat_data_message.read().as_ref().is_some_and(|message| message.is_encounter_active)
    }


    pub fn show_sample_data(&self, message: CombatDataMessage) {
        let mut combat_data_message = self.combat_data_message;
        combat_data_message.set(Some(Rc::new(message)));
    }
}

/// Where a parsed message goes: straight into the app state, from the socket task (no effect round trip).
#[derive(Clone, Copy)]
struct MessageSink {
    actions: AppActions,
    local_player_name: Signal<String>,
    view: SettingsView,
}

/// Owns the connection to ACT for the whole page: creates the context, runs the task that keeps it open, and feeds what arrives
/// into the page (the combat tables' data, the local player's name). Everything below it can read `OverlayPluginContext`.
#[component]
pub fn ActConnection(children: Element) -> Element {
    let actions = use_context::<AppActions>();
    let tables = use_context::<TablesContext>();
    let view = use_context::<SettingsView>();

    let connection_status = use_signal(|| ConnectionStatus::NotConfigured);
    let combat_data_message = use_signal(|| None);
    let player_name = use_signal(String::new);
    let error = use_signal(|| None);

    use_context_provider(|| OverlayPluginContext::new(connection_status, combat_data_message, error));
    let sink = MessageSink { actions, local_player_name: tables.local_player_name, view };
    spawn_connection_task(connection_status, combat_data_message, player_name, error, sink);

    children
}

fn spawn_connection_task(
    connection_status: Signal<ConnectionStatus>,
    combat_data_message: Signal<Option<Rc<CombatDataMessage>>>,
    player_name: Signal<String>,
    error: Signal<Option<String>>,
    sink: MessageSink,
) {
    dioxus::hooks::use_future(move || keep_connected(connection_status, combat_data_message, player_name, error, sink));
}

async fn keep_connected(
    mut connection_status: Signal<ConnectionStatus>,
    combat_data_message: Signal<Option<Rc<CombatDataMessage>>>,
    player_name_signal: Signal<String>,
    mut error_signal: Signal<Option<String>>,
    sink: MessageSink,
) {
    let url = match parse_overlay_plugin_url() {
        Ok(url) => url,
        Err(error) => {
            connection_status.set(ConnectionStatus::NotConfigured);
            error_signal.set(Some(error.to_string()));
            return;
        }
    };

    let mut failed_attempts: u32 = 0;
    loop {
        let (open_tx, mut open_rx) = mpsc::unbounded();
        let (message_tx, mut message_rx) = mpsc::unbounded();
        let (error_tx, mut error_rx) = mpsc::unbounded();
        let (close_tx, mut close_rx) = mpsc::unbounded();

        connection_status.set(ConnectionStatus::Connecting { attempt: failed_attempts + 1 });
        let socket = match BrowserWebsocketClient::connect(url.websocket_url(), open_tx, message_tx, error_tx, close_tx) {
            Ok(socket) => Some(socket),
            Err(error) => {
                error_signal.set(Some(StartError(error.to_string()).to_string()));
                None
            }
        };
        let mut had_opened = false;

        if let Some(socket) = &socket {
            'connected: loop {
                select_biased! {
                    open = open_rx.next() => if open.is_some() {
                        had_opened = true;
                        failed_attempts = 0;
                        if let Err(error) = socket.send(&subscribe_message()) {
                            error_signal.set(Some(error.to_string()));
                        }
                        connection_status.set(ConnectionStatus::Connected);
                    },
                    message = message_rx.next() => if let Some(text) = message {
                        handle_message(&text, combat_data_message, player_name_signal, error_signal, sink);
                    },
                    error = error_rx.next() => if let Some(error) = error {
                        error_signal.set(Some(error.to_string()));
                    },
                    _closed = close_rx.next() => break 'connected,
                }
            }
        }
        drop(socket);

        let retry_in_seconds = if had_opened {
            failed_attempts = 0;
            1
        } else {
            failed_attempts += 1;
            retry_delay_seconds(failed_attempts)
        };
        connection_status.set(ConnectionStatus::Disconnected { retry_in_seconds });
        TimeoutFuture::new(retry_in_seconds * 1_000).await;
    }
}

fn handle_message(
    text: &str,
    mut combat_data_message: Signal<Option<Rc<CombatDataMessage>>>,
    mut player_name_signal: Signal<String>,
    mut error_signal: Signal<Option<String>>,
    sink: MessageSink,
) {
    // An idle "still connected" heartbeat (no combatants, no fight, and none was running) shows nothing and is skipped.
    // An empty message is *not* skipped when a fight is running or just was: ACT starts a new encounter with active
    // messages that have no combatants yet, and the original overlay redraws (and so empties) its tables on them,
    // and treats the first inactive message after them as the end of that fight.
    // The first message of all is never skipped: it is what tells the app that ACT is connected (and idle).
    let previous = combat_data_message.peek().clone();
    let fight_was_running = previous.as_ref().is_some_and(|previous| previous.is_encounter_active);
    if previous.is_some() && peek_combat_data(text).is_some_and(|peek| !peek.has_combatants && !peek.is_active && !fight_was_running) {
        return;
    }
    let player_name = player_name_signal.peek().clone();
    let options = ParseOptions {
        merge_pets_into_owner: sink.view.page.peek().merge_pets,
        local_player_name: Some(player_name.as_str()).filter(|name| !name.is_empty()),
    };
    match interpret(text, options) {
        Ok(Some(OverlayPluginEvent::CombatData(data))) => {
            let message = Rc::new(data);
            combat_data_message.set(Some(Rc::clone(&message)));
            on_combat_data_changed(sink.actions, message);
        }
        Ok(Some(OverlayPluginEvent::PrimaryPlayerChanged(name))) => {
            if !name.is_empty() {
                let mut local_player_name = sink.local_player_name;
                local_player_name.set(name.clone());
            }
            player_name_signal.set(name);
        }
        Ok(None) => {}
        Err(error) => error_signal.set(Some(error.to_string())),
    }
}

fn parse_overlay_plugin_url() -> Result<OverlayPluginUrl, OverlayPluginUrlError> {
    let text = overlay_ws_from_page().ok_or(OverlayPluginUrlError::Websocket(
        crate::infrastructure::browser_websocket::UrlError::MissingHost("(no ?OVERLAY_WS= in the page's URL)".into()),
    ))?;
    OverlayPluginUrl::parse(&text)
}

fn overlay_ws_from_page() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(search.as_str()).ok()?;
    params.get("OVERLAY_WS").or_else(|| params.get("HOST_PORT")).filter(|url| !url.is_empty())
}
