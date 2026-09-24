//! Address box for connecting to ACT from a hosted copy of the overlay. Hidden while connected.

use crate::application::app_state::{sample_combat_message, AppContext, Screen};
use crate::infrastructure::network::{self, ConnectionStatus};
use crate::presentation::ui::app_shell::make_act_callbacks;
use dioxus::prelude::*;
use std::rc::Rc;

#[component]
pub(super) fn ConnectBox() -> Element {
    let context = use_context::<AppContext>();
    let mut typed_address = use_signal(|| network::saved_endpoint_text().unwrap_or_default());
    let status_message = match *context.connection_status.read() {
        ConnectionStatus::Connecting | ConnectionStatus::Connected => return rsx! {},
        ConnectionStatus::Idle => "Not connected — enter your OverlayPlugin WebSocket address, or view sample data.",
        ConnectionStatus::Disconnected => "Disconnected. Retrying every 5 seconds…",
    };
    let connect = move |_| {
        let address = typed_address.peek().trim().to_string();
        if !address.is_empty() {
            network::connect_to_address(&address, make_act_callbacks(context));
        }
    };
    let show_sample_data = move |_| {
        let mut displayed = context.displayed_combat_data;
        displayed.set(Some(Rc::new(sample_combat_message().clone())));
        let mut has_received_data = context.has_received_data;
        has_received_data.set(true);
        let mut screen = context.current_screen;
        screen.set(Screen::Main);
    };
    rsx! {
        div { class: "connectBox",
            div { class: "stat", "{status_message}" }
            input {
                r#type: "text",
                value: "{typed_address}",
                placeholder: "127.0.0.1:10501  or  ws://127.0.0.1:10501/ws",
                oninput: move |event| typed_address.set(event.value()),
            }
            div {
                span { class: "cbtn", onclick: connect, "Connect" }
                span { class: "cbtn alt", onclick: show_sample_data, "Show sample data" }
            }
        }
    }
}
