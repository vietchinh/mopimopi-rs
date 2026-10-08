//! The top bar: its buttons' transient state.

use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct NavigationBarContext {
    /// The hidden Capture / History / End buttons are shown (mouse over the ⋮ button).
    pub nav_buttons_expanded: Signal<bool>,
    /// The Capture button icon is blinking.
    pub capture_flash_active: Signal<bool>,
}

impl NavigationBarContext {
    /// A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let context = Self { nav_buttons_expanded: use_signal(|| false), capture_flash_active: use_signal(|| false) };
        use_context_provider(|| context)
    }
}
