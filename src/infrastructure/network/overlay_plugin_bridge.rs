//! Talking to OverlayPlugin from inside its own browser, plus the legacy DOM event.

use super::callbacks::ActEventCallbacks;
use super::connection_status::ConnectionStatus;
use super::javascript_json::{javascript_value_to_json_text, warn_unparseable_message};
use super::websocket_connection::{handle_incoming_text, send_overlay_api_request};
use crate::models::act_data::{parse_bare_combat_data, ActEvent};
use gloo_timers::callback::Timeout;
use serde_json::json;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const API_POLL_INTERVAL_MILLISECONDS: u32 = 200;
const API_POLL_ATTEMPTS: u32 = 25;
/// Length of a GUID such as `0f8fad5b-d9cb-469f-a165-70867728950e`.
const GUID_LENGTH: usize = 36;
const GUID_DASH_POSITIONS: [usize; 4] = [8, 13, 18, 23];

/// The id OverlayPlugin uses to address this overlay window: a global set by the plugin, or a
/// GUID inside the browser's user agent text.
pub(super) fn find_overlay_window_id() -> Option<String> {
    let window = web_sys::window()?;
    if let Some(id) = js_sys::Reflect::get(&window, &"overlayWindowId".into()).ok().and_then(|value| value.as_string()) {
        return Some(id);
    }
    let user_agent = window.navigator().user_agent().ok()?;
    find_guid(&user_agent)
}

fn find_guid(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    (0..bytes.len().saturating_sub(GUID_LENGTH - 1))
        .map(|start| &bytes[start..start + GUID_LENGTH])
        .find(|candidate| {
            candidate.iter().enumerate().all(|(position, byte)| {
                if GUID_DASH_POSITIONS.contains(&position) { *byte == b'-' } else { byte.is_ascii_hexdigit() }
            })
        })
        .map(|guid| String::from_utf8_lossy(guid).into_owned())
}

fn overlay_plugin_api() -> Option<JsValue> {
    let window = web_sys::window()?;
    let api = js_sys::Reflect::get(&window, &"OverlayPluginApi".into()).ok()?;
    (!api.is_undefined() && !api.is_null()).then_some(api)
}

/// Subscribes to events through `OverlayPluginApi` if it exists. Returns whether it did.
fn subscribe_through_overlay_plugin_api(callbacks: ActEventCallbacks) -> bool {
    let Some(api) = overlay_plugin_api() else { return false };
    let Some(window) = web_sys::window() else { return false };

    let callbacks_for_messages = callbacks.clone();
    let receive_message = Closure::<dyn FnMut(JsValue)>::new(move |message: JsValue| {
        if let Some(text) = javascript_value_to_json_text(&message) {
            handle_incoming_text(&text, &callbacks_for_messages);
        }
    });
    let _ = js_sys::Reflect::set(&window, &"__OverlayCallback".into(), receive_message.as_ref().unchecked_ref());
    receive_message.forget();

    let subscription = json!({"call": "subscribe", "events": ["CombatData", "ChangePrimaryPlayer"]}).to_string();
    if let Some(call_handler) = js_sys::Reflect::get(&api, &"callHandler".into()).ok().and_then(|f| f.dyn_into::<js_sys::Function>().ok()) {
        let ignore_reply = Closure::<dyn FnMut(JsValue)>::new(|_reply: JsValue| {});
        let _ = call_handler.call2(&api, &subscription.into(), ignore_reply.as_ref().unchecked_ref());
        ignore_reply.forget();
    }
    callbacks.report_status(ConnectionStatus::Connected);
    true
}

/// Inside OverlayPlugin the API object can appear a moment after the page loads, so keep
/// looking for it for a few seconds.
pub fn poll_until_overlay_plugin_api_appears(callbacks: ActEventCallbacks) {
    poll_attempt(callbacks, 0);
}

fn poll_attempt(callbacks: ActEventCallbacks, attempt: u32) {
    if subscribe_through_overlay_plugin_api(callbacks.clone()) || attempt >= API_POLL_ATTEMPTS {
        return;
    }
    Timeout::new(API_POLL_INTERVAL_MILLISECONDS, move || poll_attempt(callbacks, attempt + 1)).forget();
}

/// The original also accepted combat data as a DOM `onOverlayDataUpdate` event on the window.
pub(super) fn listen_for_legacy_dom_events(callbacks: ActEventCallbacks) {
    let Some(window) = web_sys::window() else { return };
    let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
        let Ok(custom_event) = event.dyn_into::<web_sys::CustomEvent>() else { return };
        let Some(text) = javascript_value_to_json_text(&custom_event.detail()) else { return };
        match parse_bare_combat_data(&text) {
            Ok(message) => callbacks.dispatch(ActEvent::CombatData(message)),
            Err(error) => warn_unparseable_message(&error),
        }
    });
    let _ = window.add_event_listener_with_callback("onOverlayDataUpdate", listener.as_ref().unchecked_ref());
    listener.forget();
}

/// Sends `Capture` or `RequestEnd` to ACT. Ending the encounter also calls OverlayPlugin's API.
pub fn send_overlay_request(request_kind: &str) {
    send_overlay_api_request(request_kind);
    if request_kind == "RequestEnd" {
        end_encounter_through_overlay_plugin_api();
    }
}

fn end_encounter_through_overlay_plugin_api() {
    let Some(api) = overlay_plugin_api() else { return };
    let end_encounter = js_sys::Reflect::get(&api, &"endEncounter".into()).ok().and_then(|f| f.dyn_into::<js_sys::Function>().ok());
    if let Some(end_encounter) = end_encounter {
        let _ = end_encounter.call0(&api);
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infrastructure/network/overlay_plugin_bridge.rs"]
mod tests;
