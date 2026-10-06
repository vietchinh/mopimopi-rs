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
use crate::presentation::ui::areas::SettingsView;
use crate::application::i18n::translate;
use crate::domain::translations::translations;
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

#[component]
pub fn NavigationBar(is_settings_preview: bool) -> Element {
    let context = use_context::<AppContext>();
    let view = use_context::<SettingsView>();
    let nav = view.nav.read().clone();
    let merge_pets = view.page.read().merge_pets;
    let settings = context.settings.read();

    // Same source as `CombatTables`: the settings preview always shows the built-in sample fight;
    // the real bar shows whatever is currently *displayed* (`displayed_combat_data`), not
    // necessarily the newest thing OverlayPlugin has sent. That frozen snapshot is what preserves
    // the original's quirk (see `summary_line`'s doc comment): once a fight ends, the time, target
    // and DPS summary stay exactly as they were, through every "still inactive" message that
    // follows, until a new fight actually starts.
    let combat_data = if is_settings_preview {
        Some(crate::application::app_state::sample_combat_message(merge_pets).clone())
    } else {
        context.displayed_combat_data.read().as_deref().cloned()
    };
    let (time_text, target_text, summary) = match &combat_data {
        Some(message) => (
            message.encounter.duration_text.clone(),
            message.encounter.title.clone(),
            summary_line(context, &settings, &nav.summary, &message.combatants, &message.encounter, is_settings_preview),
        ),
        // Nothing has ever arrived yet: the only case the original shows this placeholder for
        // (before its own `firstCombat` flag is ever set).
        None => (
            "00:00".to_string(),
            translate(&translations().ui_schema["NAV"]["main"]["tt"]["target"]),
            rsx! { "{translate(&translations().ui_schema[\"NAV\"][\"main\"][\"tt\"][\"rps\"])}" }),
    };
    let uses_two_lines = nav.two_lines;
    let nav_vars = nav.bar_vars();
    let (time_vars, target_vars, summary_vars) = (nav.time_vars(), nav.target_vars(), nav.summary_vars());

    // The original has two alternative layouts (2 rows / 1 row); only one is visible.
    rsx! {
        nav { "name": "main", class: "nav-bar", style: "{nav_vars}",
            table { "name": "ACT_2line", display: if !uses_two_lines { "none" },
                tbody {
                    tr {
                        td { rowspan: "2", "name": "time", class: "nav-time themed-text", style: "{time_vars}", "{time_text}" }
                        td { "name": "target", class: "nav-target nav-target-2line themed-text", style: "{target_vars}", "{target_text}" }
                    }
                    tr { td { "name": "rps", class: "nav-rps-2line themed-text", style: "{summary_vars}", {summary.clone()} } }
                }
            }
            table { "name": "ACT_1line", display: if uses_two_lines { "none" },
                tbody {
                    tr {
                        td { "name": "time", class: "nav-time themed-text", style: "{time_vars}", "{time_text}" }
                        td { "name": "target", class: "nav-target nav-target-1line themed-text", style: "{target_vars}", "{target_text}" }
                        td { "name": "rps", class: "nav-rps-1line themed-text", style: "{summary_vars}", {summary} }
                    }
                }
            }
            NavigationButtons { is_settings_preview }
        }
    }
}
