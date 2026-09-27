//! Talking to the outside world.
//!
//! * `network`  – WebSocket / OverlayPlugin connection to ACT

pub mod javascript_websocket;
pub mod browser_websocket;
pub mod act;
mod utf8_buffer;
