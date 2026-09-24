//! Saving the settings to `localStorage` without doing it on every slider tick.
//!
//! Dragging a slider changes the settings dozens of times per second, and each save would
//! serialise the whole document. Instead the save is delayed; every new change restarts the
//! delay, so only the final value is written. When the page is hidden or closed, any pending
//! change is written immediately.

use crate::domain::settings::Settings;
use dioxus::prelude::*;
use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const SAVE_DELAY_MILLISECONDS: u32 = 300;

thread_local! {
    /// The scheduled save. Replacing it drops (and so cancels) the previous one.
    static PENDING_SAVE: RefCell<Option<Timeout>> = const { RefCell::new(None) };
}

/// Saves the settings `SAVE_DELAY_MILLISECONDS` after the most recent call.
pub fn schedule_settings_save(settings: Signal<Settings>) {
    let timer = Timeout::new(SAVE_DELAY_MILLISECONDS, move || {
        settings.peek().save_to_browser();
        PENDING_SAVE.with(|pending| *pending.borrow_mut() = None);
    });
    PENDING_SAVE.with(|pending| *pending.borrow_mut() = Some(timer));
}

/// Writes pending changes right away when the page is hidden or closed (`pagehide` also fires on
/// mobile when the tab goes to the background).
pub fn register_save_on_page_hide(settings: Signal<Settings>) {
    let Some(window) = web_sys::window() else { return };
    let save_now = Closure::<dyn FnMut()>::new(move || {
        let has_pending_change = PENDING_SAVE.with(|pending| pending.borrow_mut().take().is_some());
        if has_pending_change {
            settings.peek().save_to_browser();
        }
    });
    let _ = window.add_event_listener_with_callback("pagehide", save_now.as_ref().unchecked_ref());
    save_now.forget();
}
