//! MopiMopi overlay for FFXIV / ACT, written in Rust with Dioxus.
//!
//! Layers, top to bottom (a layer only imports from the ones below it):
//! `presentation` (ui, theme) -> `application` (app_state) -> `infrastructure` (network)
//! -> `domain` (combat, formatting, settings, translations) -> `models` (act_data) -> `common`.
//!
//! See `MAINTAINING.md` for a guided tour.

mod application;
#[cfg(test)]
#[path = "../tests/unit/benchmarks.rs"]
mod benchmarks;
mod common;
mod domain;
mod infrastructure;
mod presentation;

fn main() {
    // Turns a wasm panic into a readable browser-console message with the actual Rust location and
    // message, instead of the browser only reporting an opaque "unreachable" trap.
    std::panic::set_hook(Box::new(|info| web_sys::console::error_1(&info.to_string().into())));
    dioxus::launch(presentation::ui::App);
}
