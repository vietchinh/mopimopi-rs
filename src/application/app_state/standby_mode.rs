//! Standby mode: hide the tables after some minutes without a fight.

use super::app_actions::AppActions;
use super::contexts::Screen;
use super::toast_notifications::{dismiss_toast_message, show_toast_message};
use dioxus::core::Runtime;
use dioxus::prelude::*;
use gloo_timers::callback::Timeout;
use js_sys::Date;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const MILLISECONDS_PER_MINUTE: f64 = 60_000.0;
const MINIMUM_STANDBY_MILLISECONDS: f64 = 1_000.0;
/// Largest delay a browser timer accepts.
const MAXIMUM_TIMER_MILLISECONDS: f64 = 4_000_000_000.0;

thread_local! {
    /// The one timer that watches the deadline, and the moment it is set to fire.
    static STANDBY_TIMER: RefCell<Option<(Timeout, f64)>> = const { RefCell::new(None) };
    /// When the tables go to standby (milliseconds since the epoch), moved on by every message.
    static STANDBY_DEADLINE: Cell<f64> = const { Cell::new(f64::INFINITY) };
}

/// Shows the tables again and moves the standby deadline. Call whenever something happens.
///
/// A message arrives every second, so the browser timer is not restarted each time (two calls across the wasm/JS
/// boundary): only the deadline moves, and the timer, when it fires early, sleeps again for what is left.
pub fn restart_standby_timer(actions: AppActions) {
    dismiss_toast_message(actions);

    let (standby_enabled, standby_minutes) = {
        let settings = actions.settings.settings.peek();
        (settings.option_enabled("autoHide"), settings.slider_value("autoHideTime"))
    };
    if !standby_enabled || *actions.screen.current_screen.peek() == Screen::Settings {
        STANDBY_TIMER.with(|timer| *timer.borrow_mut() = None);
        STANDBY_DEADLINE.with(|deadline| deadline.set(f64::INFINITY));
        return;
    }
    let mut is_hidden = actions.tables.is_standby_hidden;
    if *is_hidden.peek() {
        is_hidden.set(false); // only write when it changes: every write redraws the readers
    }
    let delay = (standby_minutes * MILLISECONDS_PER_MINUTE).clamp(MINIMUM_STANDBY_MILLISECONDS, MAXIMUM_TIMER_MILLISECONDS);
    let deadline = Date::now() + delay;
    STANDBY_DEADLINE.with(|slot| slot.set(deadline));
    // Keep the timer that is already running when it fires no later than the new deadline; otherwise (none, or the
    // deadline came forward because the setting was shortened) replace it.
    let keep = STANDBY_TIMER.with(|timer| timer.borrow().as_ref().is_some_and(|(_, fires_at)| *fires_at <= deadline));
    if !keep {
        arm_standby_timer(actions, deadline, Runtime::current());
    }
}

fn arm_standby_timer(actions: AppActions, fires_at: f64, runtime: Rc<Runtime>) {
    let wait = (fires_at - Date::now()).clamp(0.0, MAXIMUM_TIMER_MILLISECONDS);
    // A timer fires outside any component, but the toast's text comes from the translations, which live in the app's
    // scope (as a context): the callback has to run inside it, or Dioxus panics and the whole app stops.
    let callback_runtime = runtime.clone();
    let timer = Timeout::new(wait as u32, move || {
        STANDBY_TIMER.with(|timer| *timer.borrow_mut() = None);
        let deadline = STANDBY_DEADLINE.with(Cell::get);
        if deadline > Date::now() + 1.0 {
            if deadline.is_finite() {
                arm_standby_timer(actions, deadline, runtime.clone()); // a message came in meanwhile: sleep for the rest
            }
            return;
        }
        callback_runtime.in_scope(ScopeId::APP, || {
            let mut is_hidden = actions.tables.is_standby_hidden;
            let mut dropdown = actions.dropdown.open_dropdown;
            dropdown.set(None);
            is_hidden.set(true);
            show_toast_message(actions, "hiddenTable", 0, 3000);
        });
    });
    STANDBY_TIMER.with(|slot| *slot.borrow_mut() = Some((timer, fires_at)));
}
