//! Top bar of the main screen: encounter time, target name, DPS summary and buttons.
//! Also drawn (with `is_settings_preview`) as the sample bar on settings pages.
//!
//! * `summary_line`  – "Total DPS 0  Total HPS 0  Rank 1/1/1  `MaxHit` ..."
//! * `buttons`       – Capture, History and the ⋮ button (no End-encounter command in this backend)
//!
//! Capture only blinks the icon and shows a toast for now (`capture_screenshot`, below): the real
//! browser-side screenshot code (`screenshot.rs`, `page_screenshot.js`) draws the page to a PNG and
//! downloads it, but it isn't wired in — `mod screenshot;` is commented out, so it doesn't even
//! compile. It is a real, working implementation, just parked until the plain look-alike stub is
//! no longer wanted; uncomment the module and swap the one call in this file to bring it back.
// mod screenshot;
mod buttons;
mod summary_line;

use crate::application::app_state::AppContext;
use crate::domain::translations::{translate, translations};
use buttons::NavigationButtons;
use dioxus::prelude::*;
use summary_line::summary_line;

/// Screenshot request: blink the icon and say so in a toast. The original asked `ACTWebSocket` to
/// save a capture; that protocol is gone (see `MAINTAINING.md`), and the real replacement (drawing
/// the page to a PNG in the browser) is parked in `screenshot.rs` for later, so for now this only
/// gives the same visible feedback the button always gave, without actually saving anything.
pub fn capture_screenshot(context: AppContext) {
    let mut flashing = context.capture_flash_active;
    flashing.set(true);
    gloo_timers::callback::Timeout::new(750, move || {
        let mut flashing = context.capture_flash_active;
        flashing.set(false);
    })
    .forget();
    crate::application::app_state::show_toast_message(context, "Capture", 1500, 8000);
}

/// Layout choice "Display Type of Combatant Data": summary below the target (2 lines) or beside it.
const TWO_LINE_LAYOUT: i32 = 2;

#[component]
pub fn NavigationBar(is_settings_preview: bool) -> Element {
    let context = use_context::<AppContext>();
    let settings = context.settings.read();
    let language = settings.language_code();

    // Same source as `CombatTables`: the settings preview always shows the built-in sample fight;
    // the real bar shows whatever is currently *displayed* (`displayed_combat_data`), not
    // necessarily the newest thing OverlayPlugin has sent. That frozen snapshot is what preserves
    // the original's quirk (see `summary_line`'s doc comment): once a fight ends, the time, target
    // and DPS summary stay exactly as they were, through every "still inactive" message that
    // follows, until a new fight actually starts.
    let combat_data = if is_settings_preview {
        Some(crate::application::app_state::sample_combat_message().clone())
    } else {
        context.displayed_combat_data.read().as_deref().cloned()
    };
    let (time_text, target_text, summary) = match &combat_data {
        Some(message) => (
            message.encounter.duration_text.clone(),
            message.encounter.title.clone(),
            summary_line(context, &settings, &language, &message.combatants, &message.encounter, is_settings_preview),
        ),
        // Nothing has ever arrived yet: the only case the original shows this placeholder for
        // (before its own `firstCombat` flag is ever set).
        None => (
            "00:00".to_string(),
            translate(&translations().ui_schema["NAV"]["main"]["tt"]["target"], &language),
            rsx! { "{translate(&translations().ui_schema[\"NAV\"][\"main\"][\"tt\"][\"rps\"], &language)}" },
        ),
    };
    let uses_two_lines = settings.option_number("act") as i32 == TWO_LINE_LAYOUT;

    // The original has two alternative layouts (2 rows / 1 row); only one is visible.
    rsx! {
        nav { "name": "main",
            table { "name": "ACT_2line", style: if uses_two_lines { "" } else { "display:none" },
                tbody {
                    tr {
                        td { rowspan: "2", "name": "time", "{time_text}" }
                        td { "name": "target", "{target_text}" }
                    }
                    tr { td { "name": "rps", {summary.clone()} } }
                }
            }
            table { "name": "ACT_1line", style: if uses_two_lines { "display:none" } else { "" },
                tbody {
                    tr {
                        td { "name": "time", "{time_text}" }
                        td { "name": "target", "{target_text}" }
                        td { "name": "rps", {summary} }
                    }
                }
            }
            NavigationButtons { is_settings_preview }
        }
    }
}
