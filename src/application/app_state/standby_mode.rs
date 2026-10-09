//! Standby mode: hide the tables after some minutes without a fight.

use super::app_actions::AppActions;
use super::contexts::Screen;
use super::toast_notifications::{dismiss_toast_message, show_toast_message};
use dioxus::core::Runtime;
use dioxus::prelude::*;
use gloo_timers::callback::Timeout;
use std::cell::RefCell;

const MILLISECONDS_PER_MINUTE: f64 = 60_000.0;
const MINIMUM_STANDBY_MILLISECONDS: f64 = 1_000.0;
/// Largest delay a browser timer accepts.
const MAXIMUM_TIMER_MILLISECONDS: f64 = 4_000_000_000.0;

thread_local! {
    /// The running inactivity timer. Replacing it drops (and so cancels) the old one, so at most
    /// one timer exists no matter how many messages arrive.
    static STANDBY_TIMER: RefCell<Option<Timeout>> = const { RefCell::new(None) };
}

/// Shows the tables again and (re)starts the inactivity timer. Call whenever something happens.
pub fn restart_standby_timer(actions: AppActions) {
    STANDBY_TIMER.with(|timer| *timer.borrow_mut() = None);
    dismiss_toast_message(actions);

    let (standby_enabled, standby_minutes) = {
        let settings = actions.settings.settings.peek();
        (settings.option_enabled("autoHide"), settings.slider_value("autoHideTime"))
    };
    if !standby_enabled || *actions.screen.current_screen.peek() == Screen::Settings {
        return;
    }
    let mut is_hidden = actions.tables.is_standby_hidden;
    if *is_hidden.peek() {
        is_hidden.set(false); // only write when it changes: every write redraws the readers
    }
    let delay = (standby_minutes * MILLISECONDS_PER_MINUTE).clamp(MINIMUM_STANDBY_MILLISECONDS, MAXIMUM_TIMER_MILLISECONDS);
    // A timer fires outside any component, but the toast's text comes from the translations, which live in the app's
    // scope (as a context): the callback has to run inside it, or Dioxus panics and the whole app stops.
    let runtime = Runtime::current();
    let timer = Timeout::new(delay as u32, move || {
        runtime.in_scope(ScopeId::APP, || {
            let mut dropdown = actions.dropdown.open_dropdown;
            dropdown.set(None);
            is_hidden.set(true);
            show_toast_message(actions, "hiddenTable", 0, 3000);
        });
    });
    STANDBY_TIMER.with(|slot| *slot.borrow_mut() = Some(timer));
}
