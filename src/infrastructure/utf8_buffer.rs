//! Copies JavaScript strings into Rust through reused buffers, without `unsafe`.
//!
//! The browser's `TextEncoder.encodeInto` converts each message to UTF-8 natively,
//! into a JavaScript `Uint8Array` that is kept between messages; the bytes are then
//! copied into a new `String` and checked. Reusing the JavaScript array is what makes
//! this fast; a Rust allocation per message costs next to nothing (~0.02 us).
//!
//! `JsValue::as_string()` is also safe, but for ASCII text wasm-bindgen's glue
//! copies one character at a time in a JavaScript loop and allocates a new
//! `String` per message. Measured in V8 on ACT-sized messages (copy + UTF-8 check):
//!
//! | message                  | as_string()         | Utf8Buffer         |
//! |--------------------------|---------------------|--------------------|
//! | 24 KB, ASCII             | 64.9 us, allocates  | 4.5 us, 0 allocs   |
//! | 62 KB, ASCII             | 168.3 us, allocates | 11.7 us, 0 allocs  |
//! | 64 KB, Japanese names    | 51.8 us, 347 KB     | 65.3 us, 0 allocs  |
//!
//! For non-ASCII text `as_string()` switches to the native encoder too, so there the
//! two are close in speed and this buffer's advantage is not allocating.

use js_sys::{JsString, Uint8Array};
use std::string::FromUtf8Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    type TextEncoder;

    #[wasm_bindgen(constructor)]
    fn new() -> TextEncoder;

    /// Takes the JS string itself, unlike web-sys's binding, which takes a
    /// Rust `&str` (the other direction).
    #[wasm_bindgen(method, js_name = encodeInto)]
    fn encode_into(this: &TextEncoder, source: &JsString, destination: &Uint8Array) -> EncodeIntoResult;

    type EncodeIntoResult;

    #[wasm_bindgen(method, getter)]
    fn written(this: &EncodeIntoResult) -> u32;
}

pub struct Utf8Buffer {
    /// Browser-side scratch space for `encodeInto`, reused between messages.
    js_bytes: Uint8Array,
    encoder: TextEncoder,
}

impl Utf8Buffer {
    pub fn new() -> Self {
        Self { js_bytes: Uint8Array::new_with_length(0), encoder: TextEncoder::new() }
    }

    /// The text as an owned `String`, for sending through a channel. Only the
    /// JavaScript array is reused: that is the part that matters for speed, since a
    /// Rust allocation per message costs next to nothing (measured at ~0.02 us).
    /// For simd-json, `into_bytes()` on the result gives its bytes without a copy.
    pub fn decode_owned(&mut self, text: &JsString) -> Result<String, FromUtf8Error> {
        let written = self.encode(text);
        String::from_utf8(self.js_bytes.subarray(0, written).to_vec())
    }

    /// Encodes `text` as UTF-8 into the reused JavaScript array; returns the byte count.
    fn encode(&mut self, text: &JsString) -> u32 {
        // UTF-16 -> UTF-8 needs at most 3 bytes per code unit, so one call always
        // fits. The JS array grows in powers of two, so it is replaced rarely.
        let needed = text.length().saturating_mul(3);
        if self.js_bytes.length() < needed {
            let size = needed.checked_next_power_of_two().unwrap_or(needed);
            self.js_bytes = Uint8Array::new_with_length(size);
        }
        self.encoder.encode_into(text, &self.js_bytes).written()
    }
}

impl Default for Utf8Buffer {
    fn default() -> Self {
        Self::new()
    }
}
