//! Everything that can go wrong with a `JsWebsocketClient`.

use super::browser_websocket_url::UrlError;
use std::fmt;
use std::string::FromUtf8Error;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum BrowserWebsocketError {
    /// The URL was not accepted; see `UrlError`.
    InvalidUrl(UrlError),
    /// The browser's `error` event. Browsers deliberately give no details; a close
    /// event follows, and its code says more.
    Connection,
    /// A text message that was not valid UTF-8 (not expected; see `Utf8Buffer`).
    InvalidUtf8(FromUtf8Error),
    /// A binary frame arrived; this client only handles text.
    BinaryFrame,
    /// `send` failed, for example because the socket is not open.
    Send(String),
}

impl fmt::Display for BrowserWebsocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidUrl(error) => write!(f, "invalid WebSocket URL: {error}"),
            Self::Connection => f.write_str("WebSocket connection error"),
            Self::InvalidUtf8(error) => write!(f, "message is not valid UTF-8: {error}"),
            Self::BinaryFrame => f.write_str("unexpected binary frame"),
            Self::Send(detail) => write!(f, "send failed: {detail}"),
        }
    }
}

impl std::error::Error for BrowserWebsocketError {}

impl From<UrlError> for BrowserWebsocketError {
    fn from(error: UrlError) -> Self {
        BrowserWebsocketError::InvalidUrl(error)
    }
}

/// Browser exceptions as readable text.
pub(super) fn describe(value: &JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:?}"))
}