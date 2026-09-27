use futures::channel::mpsc::UnboundedSender;
use js_sys::JsString;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};
use crate::infrastructure::browser_websocket::browser_websocket_error::{describe, BrowserWebsocketError};
use crate::infrastructure::browser_websocket::{BrowserWebsocketUrl, UrlError};
use crate::infrastructure::utf8_buffer::Utf8Buffer;

/// Why the connection closed, as the browser reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseInfo {
    /// The WebSocket close code: 1000 is a normal close, 1006 means the connection
    /// was lost without a close frame (for example, ACT was closed).
    pub code: u16,
    pub reason: String,
    pub was_clean: bool,
}

pub struct BrowserWebsocketClient {
    socket: WebSocket,

    _on_open: Closure<dyn FnMut(Event)>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_error: Closure<dyn FnMut(Event)>,
    _on_close: Closure<dyn FnMut(CloseEvent)>,
}

impl BrowserWebsocketClient {
    pub fn connect(
        url: &BrowserWebsocketUrl,
        on_open: UnboundedSender<()>,
        on_message: UnboundedSender<String>,
        on_error: UnboundedSender<BrowserWebsocketError>,
        on_close: UnboundedSender<CloseInfo>
    ) -> Result<Self, BrowserWebsocketError> {
        let socket = WebSocket::new(url.as_str()).map_err(|e| UrlError::RejectedByBrowser {
            url: url.as_str().to_owned(),
            detail: describe(e),
        })?;
        let open = open_handler(on_open);
        let message = message_handler(on_message, on_error.clone());
        let error = error_handler(on_error);
        let close = close_handler(on_close);

        socket.set_onopen(Some(open.as_ref().unchecked_ref()));
        socket.set_onmessage(Some(message.as_ref().unchecked_ref()));
        socket.set_onerror(Some(error.as_ref().unchecked_ref()));
        socket.set_onclose(Some(close.as_ref().unchecked_ref()));

        Ok(Self { socket, _on_open: open, _on_message: message, _on_error: error, _on_close: close })
    }

    /// Sends a text message. Fails if the connection is not open yet or has closed.
    pub fn send(&self, text: &str) -> Result<(), BrowserWebsocketError> {
        self.socket.send_with_str(text).map_err(|e| BrowserWebsocketError::Send(describe(e)))
    }

    /// Starts closing the connection; `on_close` fires when it has closed.
    pub fn close(&self) {
        let _ = self.socket.close();
    }
}

impl Drop for BrowserWebsocketClient {
    fn drop(&mut self) {
        // Detach first: once the closures are freed, the socket must not call them.
        self.socket.set_onopen(None);
        self.socket.set_onmessage(None);
        self.socket.set_onerror(None);
        self.socket.set_onclose(None);
        let _ = self.socket.close();
    }
}


fn open_handler(on_open: UnboundedSender<()>) -> Closure<dyn FnMut(Event)> {
    Closure::new(move |_: Event| {
        let _ = on_open.unbounded_send(());
    })
}

/// Text frames go to `on_message` as UTF-8 (through a reused `Utf8Buffer`); binary frames
/// and invalid text go to `on_error`.
fn message_handler(
    on_message: UnboundedSender<String>,
    on_error: UnboundedSender<BrowserWebsocketError>,
) -> Closure<dyn FnMut(MessageEvent)> {
    let mut text = Utf8Buffer::new();
    Closure::new(move |event: MessageEvent| {
        let decoded = match event.data().dyn_into::<JsString>() {
            Ok(js_text) => text.decode_owned(&js_text).map_err(BrowserWebsocketError::InvalidUtf8),
            Err(_) => Err(BrowserWebsocketError::BinaryFrame),
        };
        match decoded {
            Ok(message) => {
                let _ = on_message.unbounded_send(message);
            }
            Err(error) => {
                let _ = on_error.unbounded_send(error);
            }
        }
    })
}

fn error_handler(on_error: UnboundedSender<BrowserWebsocketError>) -> Closure<dyn FnMut(Event)> {
    Closure::new(move |_: Event| {
        let _ = on_error.unbounded_send(BrowserWebsocketError::Connection);
    })
}

fn close_handler(on_close: UnboundedSender<CloseInfo>) -> Closure<dyn FnMut(CloseEvent)> {
    Closure::new(move |event: CloseEvent| {
        let _ = on_close.unbounded_send(CloseInfo { code: event.code(), reason: event.reason(), was_clean: event.was_clean() });
    })
}