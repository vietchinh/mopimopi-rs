//! `MopiMopi` overlay for FFXIV / ACT, written in Rust with Dioxus.
//!
//! Layers, top to bottom (a layer only imports from the ones below it):
//! `presentation` (ui, theme) -> `application` (`app_state`) -> `infrastructure` (act, `browser_websocket`)
//! -> `domain` (combat, formatting, settings, translations) -> `common`.
//!
//! See `MAINTAINING.md` for a guided tour.
//!
//! `clippy::pedantic` is enabled in CI. A few categories are turned back off here, crate-wide,
//! because they conflict with choices made deliberately throughout this codebase rather than
//! flagging real mistakes; everything else pedantic finds is meant to be fixed, not silenced.
//! - `large_types_passed_by_value`: `AppContext`/`OverlayPluginContext` are `Copy` structs of
//!   `Signal` handles, made that way specifically so they can be moved into `move |_|` event
//!   closures and `use_effect`/`use_future` bodies without lifetime issues. Their byte size is
//!   irrelevant — copying a `Signal` copies a handle, not the data it points to — so "pass by
//!   reference instead" is not an improvement for this type, even though it is for most 640-byte
//!   structs.
//! - `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`: every setting value
//!   (`Settings::option_number`, `slider_value`, ...) is deliberately `f64`, matching the loosely
//!   typed JSON `Settings` is stored as (mirroring the original overlay's own JS numbers), and is
//!   cast to `i32`/`u32`/`usize` at the many small comparison/indexing boundaries throughout the
//!   settings and theme code. That is expected and safe here (settings values are small, known
//!   ranges — slider steps, enum-like option numbers, row counts), not an oversight to guard with
//!   a cast wrapper at every call site.
//! - `float_cmp`: the handful of float comparisons in this codebase check a value against an
//!   exact literal a setting was just set to or reset to (e.g. a slider's un-adjusted 100%), not
//!   two independently computed floats that could differ by rounding error.
//! - `similar_names`: this app is inherently full of parallel `_dps`/`_hps` and `damage`/`healing`
//!   pairs (ACT's own vocabulary); renaming them to look less alike would make the domain harder
//!   to follow, not easier.
#![allow(
    clippy::large_types_passed_by_value,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::float_cmp,
    clippy::similar_names
)]

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
