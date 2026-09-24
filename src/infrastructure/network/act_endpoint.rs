//! Where to connect and which protocol that address speaks.

use crate::domain::settings::{read_local_storage, write_local_storage};

const SAVED_ADDRESS_STORAGE_KEY: &str = "Mopi2_ws";
const MINI_PARSE_PATH_SUFFIX: &str = "/miniparse";
const OVERLAY_PLUGIN_DEFAULT_PATH: &str = "/ws";

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ActEndpoint {
    /// Legacy MiniParse protocol (`.../MiniParse`).
    MiniParse(String),
    /// OverlayPlugin's own WebSocket protocol.
    OverlayPlugin(String),
}

impl ActEndpoint {
    pub(super) fn url(&self) -> &str {
        match self {
            ActEndpoint::MiniParse(url) | ActEndpoint::OverlayPlugin(url) => url,
        }
    }

    /// Interprets an address typed by the user: a MiniParse URL, a WebSocket URL, or a bare
    /// `host:port` (assumed to be OverlayPlugin's default `/ws` path).
    pub(super) fn from_user_text(text: &str) -> ActEndpoint {
        let address = text.trim().to_string();
        if address.to_lowercase().ends_with(MINI_PARSE_PATH_SUFFIX) {
            ActEndpoint::MiniParse(address)
        } else if address.starts_with("ws://") || address.starts_with("wss://") {
            ActEndpoint::OverlayPlugin(address)
        } else {
            ActEndpoint::OverlayPlugin(format!("ws://{}{OVERLAY_PLUGIN_DEFAULT_PATH}", address.trim_end_matches('/')))
        }
    }
}

/// `HOST_PORT` may be `host:port` or a full `ws://host:port`; the legacy path is appended.
fn mini_parse_url_from_host_port(host_port: &str) -> String {
    let host_port = host_port.trim_end_matches('/');
    if host_port.starts_with("ws://") || host_port.starts_with("wss://") {
        format!("{host_port}/MiniParse")
    } else {
        format!("ws://{host_port}/MiniParse")
    }
}

fn query_parameter(name: &str) -> Option<String> {
    let query = web_sys::window()?.location().search().ok()?;
    let parameters = web_sys::UrlSearchParams::new_with_str(&query).ok()?;
    parameters.get(name).filter(|value| !value.is_empty())
}

pub fn saved_endpoint_text() -> Option<String> {
    read_local_storage(SAVED_ADDRESS_STORAGE_KEY).filter(|text| !text.is_empty())
}

pub(super) fn remember_endpoint(address: &str) {
    write_local_storage(SAVED_ADDRESS_STORAGE_KEY, address);
}

/// The endpoint to use at startup: `?HOST_PORT=`, then `?OVERLAY_WS=`, then the saved address.
pub(super) fn discover_endpoint() -> Option<ActEndpoint> {
    if let Some(host_port) = query_parameter("HOST_PORT") {
        return Some(ActEndpoint::MiniParse(mini_parse_url_from_host_port(&host_port)));
    }
    if let Some(address) = query_parameter("OVERLAY_WS") {
        return Some(ActEndpoint::from_user_text(&address));
    }
    saved_endpoint_text().map(|address| ActEndpoint::from_user_text(&address))
}

#[cfg(test)]
#[path = "../../../tests/unit/infrastructure/network/act_endpoint.rs"]
mod tests;
