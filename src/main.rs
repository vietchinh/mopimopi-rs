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
mod models;
mod presentation;

fn main() {
    dioxus::launch(presentation::ui::App);
}
