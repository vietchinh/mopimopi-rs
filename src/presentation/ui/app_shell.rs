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

// The fonts. Their `@font-face` rules can't stay in the stylesheet: the stylesheet is served from a hashed
// `/assets/` URL, so a relative `url(font/...)` inside it would point at `/assets/font/...`. As assets the fonts get
// URLs that are correct wherever the app is hosted. They are bundled (assets/font/README.md has the licences) so the
// overlay needs no network for them; the text font and the icon font used to come from fonts.googleapis.com.
const CLOCK_FONT_WOFF: Asset = asset!("/assets/font/DS-DIGIB.woff");
const CLOCK_FONT_TTF: Asset = asset!("/assets/font/DS-DIGIB.ttf");
const TEXT_FONT_LATIN: Asset = asset!("/assets/font/RobotoCondensed-latin-400.woff2");
const TEXT_FONT_LATIN_EXT: Asset = asset!("/assets/font/RobotoCondensed-latin-ext-400.woff2");
const ICON_FONT: Asset = asset!("/assets/font/MaterialIcons-Regular.woff2");

/// Roboto Condensed (regular; bold is the browser's), split into the two subsets that cover names and the app's languages,
/// with the unicode ranges Google serves them under. Any other character falls through to the next family of the font stack.
const TEXT_FONT_LATIN_RANGE: &str = "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD";
const TEXT_FONT_LATIN_EXT_RANGE: &str = "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF";

/// The `@font-face` rules of every font, and the `.material-icons` rule that turns an icon's name into its glyph
/// (the same rule Google's icon stylesheet carries).
fn fonts_css() -> String {
    format!(
        "@font-face{{font-family:'DS-Digital';src:url({clock_woff}) format('woff'),url({clock_ttf}) format('truetype')}}\
         @font-face{{font-family:'Roboto Condensed';font-style:normal;font-weight:400;font-display:swap;src:url({latin}) format('woff2');unicode-range:{latin_range}}}\
         @font-face{{font-family:'Roboto Condensed';font-style:normal;font-weight:400;font-display:swap;src:url({latin_ext}) format('woff2');unicode-range:{latin_ext_range}}}\
         @font-face{{font-family:'Material Icons';font-style:normal;font-weight:400;src:url({icons}) format('woff2')}}\
         .material-icons{{font-family:'Material Icons';font-weight:normal;font-style:normal;font-size:24px;line-height:1;letter-spacing:normal;text-transform:none;display:inline-block;white-space:nowrap;word-wrap:normal;direction:ltr;-webkit-font-feature-settings:'liga';-webkit-font-smoothing:antialiased}}",
        clock_woff = asset_url(CLOCK_FONT_WOFF, "assets/font/DS-DIGIB.woff"),
        clock_ttf = asset_url(CLOCK_FONT_TTF, "assets/font/DS-DIGIB.ttf"),
        latin = asset_url(TEXT_FONT_LATIN, "assets/font/RobotoCondensed-latin-400.woff2"),
        latin_range = TEXT_FONT_LATIN_RANGE,
        latin_ext = asset_url(TEXT_FONT_LATIN_EXT, "assets/font/RobotoCondensed-latin-ext-400.woff2"),
        latin_ext_range = TEXT_FONT_LATIN_EXT_RANGE,
        icons = asset_url(ICON_FONT, "assets/font/MaterialIcons-Regular.woff2"),
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
    let font_face = fonts_css();
    let show_resize_handle = screen != Screen::Settings && view.page.read().corner_handle;

    rsx! {
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
