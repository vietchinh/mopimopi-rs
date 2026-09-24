//! Floating messages: the hover tooltip and the sliding toast.

use crate::application::app_state::{dismiss_toast_message, AppContext};
use crate::presentation::ui::shared::safe_markup::markup_view;
use dioxus::prelude::*;

/// Small hover hint (column descriptions, button names).
#[component]
pub fn Tooltip() -> Element {
    let context = use_context::<AppContext>();
    let text = context.tooltip_html.read().clone();
    let is_visible = text.is_some();
    let html = text.unwrap_or_default();
    rsx! {
        span {
            class: "shadow",
            id: "tooltip",
            style: if is_visible { "display:block" } else { "display:none" },
            {markup_view(&html)}
        }
    }
}

/// Slide-in message ("Backup completed"). Timing is handled by `app_state::show_toast_message`.
#[component]
pub fn Toast() -> Element {
    let context = use_context::<AppContext>();
    let toast = context.toast_message.read().clone();
    rsx! {
        div {
            class: if toast.is_slid_in { "toast jam shadow on" } else { "toast jam shadow" },
            style: if toast.is_visible { "display:block" } else { "display:none" },
            onclick: move |_| dismiss_toast_message(context),
            {markup_view(&toast.text)}
        }
    }
}
