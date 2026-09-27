use crate::infrastructure::act::data::{peek_combat_data, CombatDataMessage, CombatantRecord, EncounterRecord};
use crate::infrastructure::act::overlay_plugin_protocol::{OverlayPluginEvent, OverlayPluginUrl, OverlayPluginUrlError};
use crate::infrastructure::browser_websocket::connection_status::ConnectionStatus;
use crate::infrastructure::browser_websocket::{BrowserWebsocketClient, CloseInfo};
use dioxus::hooks::use_future;
use dioxus::prelude::Signal;
use dioxus::signals::{ReadableExt, WritableExt};
use futures::channel::mpsc;
use futures::{select_biased, StreamExt};
use gloo_timers::future::TimeoutFuture;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartError {
    /// The page has no `?OVERLAY_WS=`: open the overlay from OverlayPlugin.
    MissingUrl,
    /// `?OVERLAY_WS=` is not a valid OverlayPlugin address.
    InvalidUrl(OverlayPluginUrlError),
    /// The browser refused to start the connection.
    Connect(String),
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingUrl => f.write_str(
                "No ?OVERLAY_WS= in the page address. Open this overlay from OverlayPlugin, \
                 or add ?OVERLAY_WS=ws://127.0.0.1:10501/ws",
            ),
            Self::InvalidUrl(error) => write!(f, "Invalid ?OVERLAY_WS= address: {error}"),
            Self::Connect(error) => write!(f, "Could not connect: {error}"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct OverlayPluginContext {
    connection_status: Signal<ConnectionStatus>,
    combat_data_message: Signal<Rc<CombatDataMessage>>,
    player_name: Signal<String>,
    //pub history: ReadSignal<EncounterHistory>,
    error: Signal<String>,
    merge_player_combat_data_with_pet: Signal<bool>,
}

impl OverlayPluginContext {
    pub fn new() -> OverlayPluginContext {
        let connection_status = Signal::new(ConnectionStatus::Disconnected);
        let combat_data_message = Default::default();
        let player_name = Default::default();
        let error = Default::default();
        let merge_player_combat_data_with_pet = Signal::new(true);

        use_future(move || keep_connected(connection_status, combat_data_message, player_name, error, merge_player_combat_data_with_pet));

        OverlayPluginContext {
            connection_status,
            combat_data_message,
            player_name,
            error,
            merge_player_combat_data_with_pet
        }
    }

    pub fn get_local_player_combatant_record(&self) -> Option<CombatantRecord> {
        self.combat_data_message.read().local_player().cloned()
    }

    pub fn get_is_encounter_active(&self) -> bool {
        self.combat_data_message.read().is_encounter_active
    }

    pub fn get_combatant_records(&mut self) -> Vec<CombatantRecord> {
        self.combat_data_message.read().combatants.clone()
    }

    pub fn get_encounter_record(&mut self) -> EncounterRecord{
        self.combat_data_message.read().encounter.clone()
    }
}

// pub fn create_overlay_plugin_context() -> OverlayPluginContext {
//
//
//     use_future(move || keep_connected(connection_status, combat_data_message, player_name, error, merge_player_combat_data_with_pet));
// }

/// (common.js uses 300 ms).
const RECONNECT_DELAY_MS: u32 = 1_000;

async fn keep_connected(mut connection_status: Signal<ConnectionStatus>, mut combat_data_message: Signal<Rc<CombatDataMessage>>, mut player_name_signal: Signal<String>, mut error_signal: Signal<String>, merge_player_combat_data_with_pet: Signal<bool>) {
    let url = match parse_overlay_plugin_url() {
        Ok(url) => url,
        Err(error) => return error_signal.set(error.to_string()),
    };

    loop {
        let (open_tx, mut open_rx) = mpsc::unbounded();
        let (message_tx, mut message_rx) = mpsc::unbounded();
        let (error_tx, mut error_rx) = mpsc::unbounded();
        let (close_tx, mut close_rx) = mpsc::unbounded();

        let socket = match BrowserWebsocketClient::connect(url.websocket_url(), open_tx, message_tx, error_tx, close_tx) {
            Ok(opened) => opened,
            Err(error) => return error_signal.set(StartError::Connect(error.to_string()).to_string()),
        };

        connection_status.set(ConnectionStatus::Connecting);

        loop {
            select_biased! {
                open = open_rx.next() => if open.is_some() {
                    if let Err(error) = socket.send(r#"{"call":"subscribe","events":["CombatData","ChangePrimaryPlayer"]}"#) {
                        error_signal.set(error.to_string());
                        break;
                    }

                    connection_status.set(ConnectionStatus::Connected);
                },
                message = message_rx.next() => if let Some(message) = message {
                    if peek_combat_data(message.as_str()).is_some_and(|peek| !peek.has_combatants) {
                        return
                    }

                    match crate::infrastructure::act::overlay_plugin_protocol::interpret(message.as_str()) {
                        Ok(event) => {
                            match event {
                                Some(OverlayPluginEvent::CombatData(combat_data)) => {
                                    combat_data_message.set(Rc::new(combat_data));
                                },
                                Some(OverlayPluginEvent::PrimaryPlayerChanged(player_name)) => {
                                    player_name_signal.set(player_name);
                                },
                                None => {}
                            }

                        }
                        Err(error) => {
                            error_signal.set(error.to_string());
                        }
                    }
                },
                error = error_rx.next() => if let Some(error) = error {
                    error_signal.set(error.to_string());
                },
                close = close_rx.next() => {
                    connection_status.set(ConnectionStatus::Disconnected);
                    break
                },
            }
        }

        drop(socket);
        TimeoutFuture::new(RECONNECT_DELAY_MS).await;
    }
}

fn parse_overlay_plugin_url() -> Result<OverlayPluginUrl, StartError> {
    let text = overlay_ws_from_page().ok_or(StartError::MissingUrl)?;
    OverlayPluginUrl::parse(&text).map_err(StartError::InvalidUrl)
}

fn overlay_ws_from_page() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    web_sys::UrlSearchParams::new_with_str(search.as_str())
        .ok()?
        .get("OVERLAY_WS")
        .filter(|url| !url.is_empty())
}

fn warn_closed(info: Option<&CloseInfo>) {
    if let Some(info) = info {
        warn(&format!("OverlayPlugin connection closed: code {} {}", info.code, info.reason));
    }
}

fn warn(message: &str) {
    web_sys::console::warn_1(&message.into());
}