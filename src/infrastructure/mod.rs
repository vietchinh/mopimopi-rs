//! Talking to the outside world.
//!
//! * `browser_storage` – settings and backup in `localStorage`
//! * `network`  – WebSocket / `OverlayPlugin` connection to ACT

pub mod browser_storage;
pub mod browser_websocket;
pub mod act;
mod utf8_buffer;
