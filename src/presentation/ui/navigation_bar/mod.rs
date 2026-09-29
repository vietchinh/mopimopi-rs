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
use crate::domain::settings::Settings;
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use crate::presentation::ui::shared::color_conversion::rgba;
use crate::presentation::ui::shared::style_values::StyleValues;
use buttons::NavigationButtons;
use dioxus::prelude::*;
use summary_line::summary_line;

/// Background: plain colour or one of five patterns drawn over it.
fn background_rule(values: &StyleValues) -> String {
    let pattern = rgba(&values.color("pattern"), values.opacity("pattern"));
    let base = rgba(&values.color("navBg"), values.opacity("navBg"));
    match values.text("pattern").as_str() {
        "cross" => format!(
            "-webkit-linear-gradient({pattern},transparent .1rem),-webkit-linear-gradient(0,{pattern},{base} .1rem)"
        ),
        "hStripe" => format!("-webkit-linear-gradient({pattern},transparent .1rem),-webkit-linear-gradient(0,{pattern},{base} 0)"),
        "vStripe" => format!("-webkit-linear-gradient({pattern},transparent 0),-webkit-linear-gradient(0,{pattern},{base} .1rem)"),
        "leftDig" => format!("repeating-linear-gradient(45deg,{pattern} 0,{pattern} 5%,{base} 0,{base} 50%) 0"),
        "rightDig" => format!("repeating-linear-gradient(135deg,{pattern} 0,{pattern} 5%,{base} 0,{base} 50%) 0"),
        _ => base,
    }
}

/// The migrated equivalent of "padding that hides an element by giving it no size when its
/// opacity slider is 0": both the padding and font-size resolve to the hidden case's values up
/// front, rather than relying on a later CSS declaration overriding an earlier one.
fn hidden_or_indented(opacity_slider_key: &str, real_size: String, values: &StyleValues) -> (String, String) {
    if values.slider(opacity_slider_key) == 0.0 { ("0".to_string(), "0".to_string()) } else { (real_size, "1rem".to_string()) }
}

/// The nav bar's own style variables (background, border, icon colour/size, time / target / rps
/// text, layout gap) -- set once, on the `nav` element itself. Every descendant (icons in
/// `buttons.rs`, the time/target/rps cells below) inherits these through the DOM tree via CSS's
/// normal cascade, so nothing else needs to redeclare them. Shared between this screen's own nav
/// and the History screen's nav (`history_screen/mod.rs`), which needs the same styling for a
/// completely separate `<nav>` element.
pub(crate) fn nav_bar_style(settings: &Settings) -> String {
    let values = StyleValues::new(settings);
    let btn_wrap_bg = if values.slider("navBg") != 100.0 { "transparent".to_string() } else { "unset".to_string() };
    let (border, edge_width) = if values.slider("edge") != 0.0 {
        let width = values.slider_as_rem("sizeEdge");
        (format!("{width} {} {}", values.text("edgeType"), values.color_with_opacity("edge", "edge")), width)
    } else {
        ("unset".to_string(), "unset".to_string())
    };
    let [radius_tl, radius_tr, radius_bl, radius_br] = values.corner_radius("rd_nav", "sizeRadius");
    let (time_size, time_padding) = hidden_or_indented("navTime", values.slider_as_rem("sizeTime"), &values);
    let (target_size, target_padding) = hidden_or_indented("target", values.slider_as_rem("sizeTarget"), &values);
    let style = format!(
        "--nav-bg:{};--nav-height:{};--nav-pattern-size:{};\
         --nav-radius-tl:{radius_tl};--nav-radius-tr:{radius_tr};--nav-radius-bl:{radius_bl};--nav-radius-br:{radius_br};\
         --nav-more-radius-tr:{radius_tr};--nav-more-radius-br:{radius_br};--nav-icon-size:{};\
         --nav-border:{border};--nav-btn-wrap-bg:{btn_wrap_bg};--nav-edge-width:{edge_width};\
         --nav-icon-color:{};\
         --nav-time-color:{};--nav-time-font:{};--nav-time-size:{time_size};--nav-time-style:{};--nav-time-padding:{time_padding};\
         --nav-target-color:{};--nav-target-font:{};--nav-target-size:{target_size};--nav-target-style:{};--nav-target-padding:{target_padding};\
         --nav-rps-color:{};--nav-rps-font:{};--nav-rps-size:{};--nav-rps-style:{};--nav-gap:{}",
        background_rule(&values),
        values.slider_as_rem("sizeNav"),
        values.slider_as_rem("sizePattern"),
        values.slider_as_rem("sizeIcon"),
        values.color_with_opacity("accent", "navIcon"),
        values.color_with_opacity("accent", "navTime"),
        values.font_stack("fTime", "'DS-Digital', 'sans-serif'"),
        values.italic_or_normal("time_italic"),
        values.color_with_opacity("target", "target"),
        values.font_stack("fTarget", "'Segoe UI', 'sans-serif'"),
        values.italic_or_normal("target_italic"),
        values.color_with_opacity("rps", "rps"),
        values.font_stack("fRPS", "'Roboto Condensed', 'Segoe UI', 'sans-serif'"),
        values.slider_as_rem("sizeRPS"),
        values.italic_or_normal("rps_italic"),
        values.slider_as_rem("sizeGap"),
    );
    style
}

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

    // Same source as `CombatTables`: the settings preview always shows the built-in sample fight;
    // the real bar shows whatever is currently *displayed* (`displayed_combat_data`), not
    // necessarily the newest thing OverlayPlugin has sent. That frozen snapshot is what preserves
    // the original's quirk (see `summary_line`'s doc comment): once a fight ends, the time, target
    // and DPS summary stay exactly as they were, through every "still inactive" message that
    // follows, until a new fight actually starts.
    let combat_data = if is_settings_preview {
        Some(crate::application::app_state::sample_combat_message(settings.option_enabled("pets")).clone())
    } else {
        context.displayed_combat_data.read().as_deref().cloned()
    };
    let (time_text, target_text, summary) = match &combat_data {
        Some(message) => (
            message.encounter.duration_text.clone(),
            message.encounter.title.clone(),
            summary_line(context, &settings, &message.combatants, &message.encounter, is_settings_preview),
        ),
        // Nothing has ever arrived yet: the only case the original shows this placeholder for
        // (before its own `firstCombat` flag is ever set).
        None => (
            "00:00".to_string(),
            translate(&translations().ui_schema["NAV"]["main"]["tt"]["target"]),
            rsx! { "{translate(&translations().ui_schema[\"NAV\"][\"main\"][\"tt\"][\"rps\"])}" }),
    };
    let uses_two_lines = settings.option_number("act") as i32 == TWO_LINE_LAYOUT;
    let nav = nav_bar_style(&settings);

    // The original has two alternative layouts (2 rows / 1 row); only one is visible.
    rsx! {
        nav { "name": "main", class: "nav-bar", style: "{nav}",
            table { "name": "ACT_2line", display: if !uses_two_lines { "none" },
                tbody {
                    tr {
                        td { rowspan: "2", "name": "time", class: "nav-time", "{time_text}" }
                        td { "name": "target", class: "nav-target nav-target-2line", "{target_text}" }
                    }
                    tr { td { "name": "rps", class: "nav-rps nav-rps-2line", {summary.clone()} } }
                }
            }
            table { "name": "ACT_1line", display: if uses_two_lines { "none" },
                tbody {
                    tr {
                        td { "name": "time", class: "nav-time", "{time_text}" }
                        td { "name": "target", class: "nav-target nav-target-1line", "{target_text}" }
                        td { "name": "rps", class: "nav-rps nav-rps-1line", {summary} }
                    }
                }
            }
            NavigationButtons { is_settings_preview }
        }
    }
}
