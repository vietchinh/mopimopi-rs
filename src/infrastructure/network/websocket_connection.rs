//! The WebSocket to ACT: opening it, protocol handshakes, keep-alive and automatic reconnects.

use super::act_endpoint::ActEndpoint;
use super::callbacks::ActEventCallbacks;
use super::connection_status::ConnectionStatus;
use super::javascript_json::warn_unparseable_message;
use super::overlay_plugin_bridge::find_overlay_window_id;
use crate::models::act_data::parse_incoming_message;
use gloo_timers::callback::Timeout;
use serde_json::{json, Value};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

const RECONNECT_DELAY_MILLISECONDS: u32 = 5_000;
/// MiniParse sends "." as a keep-alive and expects "." back.
const KEEP_ALIVE_TEXT: &str = ".";

type EventListener = Closure<dyn FnMut(JsValue)>;

/// The open socket and the closures that must stay alive as long as it does.
struct OpenConnection {
    socket: WebSocket,
    _listeners: Vec<EventListener>,
}

thread_local! {
    static CURRENT_CONNECTION: RefCell<Option<OpenConnection>> = const { RefCell::new(None) };
    static CURRENT_ENDPOINT: RefCell<Option<ActEndpoint>> = const { RefCell::new(None) };
}

/// Remembers the endpoint (for reconnects) and connects to it.
pub(super) fn connect_and_remember(endpoint: ActEndpoint, callbacks: ActEventCallbacks) {
    CURRENT_ENDPOINT.with(|current| *current.borrow_mut() = Some(endpoint.clone()));
    open_socket(endpoint, callbacks);
}

/// Closes the current connection (without triggering a reconnect) and connects elsewhere.
pub(super) fn replace_connection(endpoint: ActEndpoint, callbacks: ActEventCallbacks) {
    CURRENT_CONNECTION.with(|current| {
        if let Some(old) = current.borrow_mut().take() {
            old.socket.set_onclose(None);
            let _ = old.socket.close();
        }
    });
    connect_and_remember(endpoint, callbacks);
}

/// Sends a request to ACT's overlay API (`Capture`, `RequestEnd`) over the open socket.
pub(super) fn send_overlay_api_request(request_kind: &str) {
    CURRENT_CONNECTION.with(|current| {
        if let Some(connection) = current.borrow().as_ref() {
            let request = json!({
                "type": "overlayAPI",
                "to": find_overlay_window_id(),
                "msgtype": request_kind,
                "msg": Value::Null,
            });
            let _ = connection.socket.send_with_str(&request.to_string());
        }
    });
}

fn open_socket(endpoint: ActEndpoint, callbacks: ActEventCallbacks) {
    callbacks.report_status(ConnectionStatus::Connecting);
    let Ok(socket) = WebSocket::new(endpoint.url()) else {
        callbacks.report_status(ConnectionStatus::Disconnected);
        schedule_reconnect(callbacks);
        return;
    };
    let listeners = vec![
        install_open_listener(&socket, endpoint, callbacks.clone()),
        install_message_listener(&socket, callbacks.clone()),
        install_close_listener(&socket, callbacks),
    ];
    CURRENT_CONNECTION.with(|current| *current.borrow_mut() = Some(OpenConnection { socket, _listeners: listeners }));
}

/// After connecting, tell ACT what we want: OverlayPlugin needs a subscription, MiniParse the window id.
fn install_open_listener(socket: &WebSocket, endpoint: ActEndpoint, callbacks: ActEventCallbacks) -> EventListener {
    let socket_for_handshake = socket.clone();
    let listener = EventListener::new(move |_event: JsValue| {
        callbacks.report_status(ConnectionStatus::Connected);
        match &endpoint {
            ActEndpoint::OverlayPlugin(_) => {
                let subscription = json!({"call": "subscribe", "events": ["CombatData", "ChangePrimaryPlayer"]});
                let _ = socket_for_handshake.send_with_str(&subscription.to_string());
            }
            ActEndpoint::MiniParse(_) => {
                if let Some(window_id) = find_overlay_window_id() {
                    let _ = socket_for_handshake.send_with_str(&json!({"type": "set_id", "id": window_id}).to_string());
                }
            }
        }
    });
    socket.set_onopen(Some(listener.as_ref().unchecked_ref()));
    listener
}

fn install_message_listener(socket: &WebSocket, callbacks: ActEventCallbacks) -> EventListener {
    let socket_for_keep_alive = socket.clone();
    let listener = EventListener::new(move |event: JsValue| {
        let Ok(event) = event.dyn_into::<MessageEvent>() else { return };
        let Some(text) = event.data().as_string() else { return };
        if text == KEEP_ALIVE_TEXT {
            let _ = socket_for_keep_alive.send_with_str(KEEP_ALIVE_TEXT);
        } else {
            handle_incoming_text(&text, &callbacks);
        }
    });
    socket.set_onmessage(Some(listener.as_ref().unchecked_ref()));
    listener
}

/// Browsers always fire `close` after an error too, so this single listener covers both.
fn install_close_listener(socket: &WebSocket, callbacks: ActEventCallbacks) -> EventListener {
    let listener = EventListener::new(move |_event: JsValue| {
        callbacks.report_status(ConnectionStatus::Disconnected);
        schedule_reconnect(callbacks.clone());
    });
    socket.set_onclose(Some(listener.as_ref().unchecked_ref()));
    listener
}

/// Parses one text message with serde and hands the resulting event to the application.
pub(super) fn handle_incoming_text(text: &str, callbacks: &ActEventCallbacks) {
    match parse_incoming_message(text) {
        Ok(Some(event)) => callbacks.dispatch(event),
        Ok(None) => {}
        Err(error) => warn_unparseable_message(&error),
    }
}

fn schedule_reconnect(callbacks: ActEventCallbacks) {
    Timeout::new(RECONNECT_DELAY_MILLISECONDS, move || {
        if let Some(endpoint) = CURRENT_ENDPOINT.with(|current| current.borrow().clone()) {
            open_socket(endpoint, callbacks);
        }
    })
    .forget();
}
