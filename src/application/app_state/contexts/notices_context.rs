//! Floating messages: the sliding toast and the hover tooltip.

use dioxus::prelude::*;

/// The message currently sliding in or out.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ToastState {
    pub text: String,
    pub is_visible: bool,
    /// True while the message is slid into view (drives the CSS transition).
    pub is_slid_in: bool,
}

#[derive(Clone, Copy)]
pub struct NoticesContext {
    pub toast_message: Signal<ToastState>,
    /// Incremented for each new message so old timers can tell they are outdated. No component reads it, so changing
    /// it never redraws anything.
    pub toast_generation: Signal<u32>,
    pub tooltip_html: Signal<Option<String>>,
}

impl NoticesContext {
    /// A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let context = Self {
            toast_message: use_signal(ToastState::default),
            toast_generation: use_signal(|| 0u32),
            tooltip_html: use_signal(|| None::<String>),
        };
        use_context_provider(|| context)
    }
}
