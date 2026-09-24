//! Connection to ACT.
//!
//! The overlay can receive combat data in four ways (all end up as `ActEvent`s, see `act_data`):
//! * `?HOST_PORT=ws://127.0.0.1:10501` – legacy MiniParse WebSocket
//! * `?OVERLAY_WS=ws://127.0.0.1:10501/ws` – OverlayPlugin WebSocket
//! * an address saved from the start screen's connect box
//! * running inside OverlayPlugin's own browser (CEF), or receiving the legacy DOM event
//!
//! Files:
//! * `act_endpoint`            – which address to use and which protocol it speaks
//! * `websocket_connection`    – opening the socket, reconnecting, sending requests
//! * `overlay_plugin_bridge`   – the in-game browser API and the legacy DOM event
//! * `callbacks`               – how events are handed to the application
//! * `connection_status`       – idle / connecting / connected / disconnected
//! * `javascript_json`         – converting JavaScript values to JSON text

mod act_endpoint;
mod callbacks;
mod connection_status;
mod javascript_json;
mod overlay_plugin_bridge;
mod websocket_connection;

pub use act_endpoint::saved_endpoint_text;
pub use callbacks::ActEventCallbacks;
pub use connection_status::ConnectionStatus;
pub use overlay_plugin_bridge::{poll_until_overlay_plugin_api_appears, send_overlay_request};

use act_endpoint::{discover_endpoint, remember_endpoint, ActEndpoint};

/// Starts listening. Call once at startup.
pub fn start_listening(callbacks: ActEventCallbacks) {
    overlay_plugin_bridge::listen_for_legacy_dom_events(callbacks.clone());
    if let Some(endpoint) = discover_endpoint() {
        websocket_connection::connect_and_remember(endpoint, callbacks);
        return;
    }
    poll_until_overlay_plugin_api_appears(callbacks.clone());
    callbacks.report_status(ConnectionStatus::Idle);
}

/// Connects to an address typed by the user (and remembers it for next time).
pub fn connect_to_address(address: &str, callbacks: ActEventCallbacks) {
    remember_endpoint(address);
    websocket_connection::replace_connection(ActEndpoint::from_user_text(address), callbacks);
}
