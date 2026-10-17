//! Which full screen is shown.

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Main,
    History,
    Settings,
}

#[derive(Clone, Copy)]
pub struct ScreenContext {
    pub current_screen: Signal<Screen>,
}

impl ScreenContext {
    /// A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let current_screen = use_signal(|| Screen::Main);
        use_context_provider(|| Self { current_screen })
    }
}
