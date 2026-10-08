//! Floating messages: the hover tooltip and the sliding toast.

use crate::application::app_state::{dismiss_toast_message, AppActions, NoticesContext};
use crate::presentation::ui::shared::safe_markup::markup_view;
use dioxus::prelude::*;

/// Small hover hint (column descriptions, button names).
#[component]
pub fn Tooltip() -> Element {
    let notices = use_context::<NoticesContext>();
    let text = notices.tooltip_html.read().clone();
    let is_visible = text.is_some();
    let html = text.unwrap_or_default();
    rsx! {
        span {
            class: "shadow",
            id: "tooltip",
            display: if is_visible { "block" } else { "none" },
            {markup_view(&html)}
        }
    }
}

/// Slide-in message ("Backup completed"). Timing is handled by `app_state::show_toast_message`.
#[component]
pub fn Toast() -> Element {
    let notices = use_context::<NoticesContext>();
    let actions = use_context::<AppActions>();
    let toast = notices.toast_message.read().clone();
    rsx! {
        div {
            class: if toast.is_slid_in { "toast jam shadow on" } else { "toast jam shadow" },
            display: if toast.is_visible { "block" } else { "none" },
            onclick: move |_| dismiss_toast_message(actions),
            {markup_view(&toast.text)}
        }
    }
}
