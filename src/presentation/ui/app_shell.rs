//! Root component: builds the state the sections share, and the page shell around the current screen.

use super::dropdown_menus::DropdownMenu;
use super::history_screen::{HistoryNavigationBar, HistoryScreen};
use super::navigation_bar::NavigationBar;
use super::areas::SettingsView;
use super::overlay_plugin_context::ActConnection;
use super::page_services::{LanguageSync, SettingsSaver, StandbyTimer, TooltipReset};
use super::overlays::{Toast, Tooltip};
use super::settings_screens::{ColorPickerHost, ColorPickerState, SettingsNavigationBar, SettingsScreen};
use super::combat_tables::BarHistory;
use super::start_screen::MainScreen;
use crate::application::app_state::{AppActions, DropdownContext, NavigationBarContext, Screen, ScreenContext, SettingsContext};
use dioxus::prelude::*;

// tailwind.css is a Dioxus asset, so `dx` minifies it and gives it a content-hashed file name (long-term
// cacheable). It is the generated part of the CSS: the Preflight undo, the utilities, and the parts of the
// original overlay's stylesheet that do not paint on a refresh (menus, settings screens, forms, tooltip).
//
// The base layout is NOT here. It is `public/base.css`, a static, render-blocking `<link>` in the page's head
// (`style` in Dioxus.toml, `<link>` in web/index.html), because a `document::Stylesheet` only reaches the page
// after the wasm has run and drawn its first frame, and until then that frame has no app CSS at all: the page
// bounced on every refresh. Bare local paths in `[web.resource] style` are not processed by `dx`, which is why
// this one is still a runtime asset: it needs the hashing, and a static file cannot have it. The layer order
// (app < legacy < utilities) is what keeps the two files' rules in the right order; the header of tailwind.css
// has the whole picture.
// Generated from `tailwind.css` at the project root: by `dx` (it writes this file itself) or by
// `build.sh`.
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// The URL of an asset. Under `dx` that is the processed file (minified and content-hashed for CSS,
/// prefixed with the app's base path). A plain `cargo` build (`build.sh`) has no `dx` to process
/// assets, so `asset!` yields a placeholder instead of a path; then the plain file that `build.sh`
/// copies next to the page is used, relative so that hosting under a sub-path still works.
fn asset_url(asset: Asset, plain_file: &'static str) -> String {
    let url = asset.to_string();
    if url.contains(manganis::BundledAsset::PLACEHOLDER_HASH) { plain_file.to_string() } else { url }
}

// The clock font. Its `@font-face` can't stay in the stylesheet: the stylesheet is served from a hashed
// `/assets/` URL, so a relative `url(font/...)` inside it would point at `/assets/font/...`. As an
// asset the font gets a URL that is correct wherever the app is hosted.
const CLOCK_FONT_WOFF: Asset = asset!("/assets/font/DS-DIGIB.woff");
const CLOCK_FONT_TTF: Asset = asset!("/assets/font/DS-DIGIB.ttf");

fn clock_font_face() -> String {
    format!(
        "@font-face{{font-family:'DS-Digital';src:url({}) format('woff'),url({}) format('truetype')}}",
        asset_url(CLOCK_FONT_WOFF, "assets/font/DS-DIGIB.woff"),
        asset_url(CLOCK_FONT_TTF, "assets/font/DS-DIGIB.ttf"),
    )
}

/// The root. It creates the state every section shares (the contexts, `AppActions`, the settings view, the translations) and
/// nothing else: each section sets up what only it needs (the colour picker, the bars' memory, the top bar's buttons), and the
/// jobs that run in the background are components of their own (see `page_services`).
#[component]
pub fn App() -> Element {
    // One context per section of the page, each provided here; `actions` is what changes several of them at once.
    AppActions::provide();
    let settings_context = use_context::<SettingsContext>();
    // Each area of the settings as a typed value, for the components that draw it (see `areas`). Created first: what is set up below
    // already needs the page's settings.
    let view = super::areas::provide_settings_view(settings_context.settings);
    // Text in the language of the `Lang` setting (see application::i18n); `LanguageSync` keeps it following the setting.
    crate::application::i18n::use_init_translations(&view.page.peek().language_code);

    rsx! {
        LanguageSync {}
        SettingsSaver {}
        StandbyTimer {}
        TooltipReset {}
        ActConnection {
            ColorPickerHost {
                PageShell {}
            }
        }
    }
}

/// The page itself: its stylesheets and page-wide variables, the open pop-up menu, the top bar and the body of the current screen.
#[component]
fn PageShell() -> Element {
    let screen_context = use_context::<ScreenContext>();
    let dropdown_context = use_context::<DropdownContext>();
    let color_picker = use_context::<ColorPickerState>();
    let view = use_context::<SettingsView>();
    // Shared by every top bar (main, history, settings preview), so it lives as long as the page does: a button's timer may fire
    // after the bar that started it has gone.
    NavigationBarContext::provide();
    // Where each graph bar was last drawn, so the next change of a bar can be animated from there.
    use_context_provider(BarHistory::default);

    // Page-wide variables (font size, background image, accent colour): owned here since
    // app_shell is the page's actual root, and `:root` in an HTML document *is* the `<html>`
    // element -- there's no other `rsx!` element this could attach to. Rebuilt only when the
    // settings change, not when the screen or a dropdown does. Everything else styling-related
    // is computed inline by whichever component owns the element it styles; this is the one
    // exception, because nothing else in the render tree is html's actual owner.
    let root_style = use_memo(move || view.page.read().root_rule());
    let screen = *screen_context.current_screen.read();
    let font_face = clock_font_face();
    let show_resize_handle = screen != Screen::Settings && view.page.read().corner_handle;

    rsx! {
        // The Material Icons font (Google's stylesheet), from dioxus-material-icons.
        dioxus_material_icons::MaterialIconStylesheet {}
        document::Stylesheet { href: asset_url(TAILWIND_CSS, "assets/tailwind.css") }
        style { "{font_face}" }
        style { "{root_style}" }
        // "Resizing Arrow" (`arrow`): the original sets the handle image on #wrap at start-up and when it
        // returns from the settings, and clears it when the settings open (ui.js), so: on unless in settings.
        div {
            id: "wrap",
            background_image: if show_resize_handle { "url(images/handle.svg)" },
            // Pressing anywhere but on the picker or its boxes closes the picker (they stop the event).
            onmousedown: move |_| color_picker.hide(),
            if dropdown_context.open_dropdown.read().is_some() {
                DropdownMenu {}
                div {
                    id: "blackBg",
                    onclick: move |_| {
                        let mut dropdown = dropdown_context.open_dropdown;
                        dropdown.set(None);
                    },
                }
            }
            match screen {
                Screen::Main => rsx! { NavigationBar { is_settings_preview: false } },
                Screen::History => rsx! { HistoryNavigationBar {} },
                Screen::Settings => rsx! { SettingsNavigationBar {} },
            }
            div { id: "content",
                Tooltip {}
                Toast {}
                match screen {
                    Screen::Main => rsx! { MainScreen {} },
                    Screen::History => rsx! { HistoryScreen {} },
                    Screen::Settings => rsx! { SettingsScreen {} },
                }
            }
        }
    }
}
