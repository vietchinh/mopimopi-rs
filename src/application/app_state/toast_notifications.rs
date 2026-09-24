//! Short messages that slide in from the right ("Backup completed").

use super::app_context::AppContext;
use crate::domain::translations::translations;
use dioxus::prelude::*;
use gloo_timers::callback::Timeout;

/// Time the slide-out animation needs before the message can be removed.
const SLIDE_OUT_MILLISECONDS: u32 = 200;

/// Shows the translated message `message_id` after `delay_milliseconds` for
/// `visible_until_milliseconds` (measured from now). Does nothing when toasts are disabled.
pub fn show_toast_message(context: AppContext, message_id: &str, delay_milliseconds: u32, visible_until_milliseconds: u32) {
    let (toasts_enabled, language) = {
        let settings = context.settings.peek();
        (settings.option_enabled("toast"), settings.language_code())
    };
    if !toasts_enabled {
        return;
    }
    let text = translations().message(message_id, &language);
    let mut toast = context.toast_message;
    if toast.peek().is_slid_in {
        toast.write().is_slid_in = false;
    }
    let generation_signal = context.toast_generation;
    let generation = start_new_message_generation(generation_signal);

    // Timers of an older message must not touch a newer one, hence the generation checks.
    Timeout::new(delay_milliseconds, move || {
        if *generation_signal.peek() == generation {
            let mut state = toast.write();
            state.text = text;
            state.is_visible = true;
            state.is_slid_in = true;
        }
    })
    .forget();
    Timeout::new(visible_until_milliseconds, move || {
        if *generation_signal.peek() == generation {
            toast.write().is_slid_in = false;
            Timeout::new(SLIDE_OUT_MILLISECONDS, move || {
                if *generation_signal.peek() == generation {
                    toast.write().is_visible = false;
                }
            })
            .forget();
        }
    })
    .forget();
}

/// Removes the current message immediately and cancels its timers. Called for every combat
/// message, so it only writes the toast state when a message is actually showing.
pub fn dismiss_toast_message(context: AppContext) {
    start_new_message_generation(context.toast_generation);
    let mut toast = context.toast_message;
    let is_showing = {
        let state = toast.peek();
        state.is_visible || state.is_slid_in
    };
    if is_showing {
        let mut state = toast.write();
        state.is_slid_in = false;
        state.is_visible = false;
    }
}

/// Invalidates every pending timer and returns the new generation number.
fn start_new_message_generation(mut generation: Signal<u32>) -> u32 {
    *generation.write() += 1;
    *generation.peek()
}
