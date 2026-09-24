//! JavaScript values arriving from the browser, as JSON text for serde.

use wasm_bindgen::JsValue;

/// A JSON string stays as it is; any other JavaScript value is serialised with `JSON.stringify`.
pub(super) fn javascript_value_to_json_text(value: &JsValue) -> Option<String> {
    if let Some(text) = value.as_string() {
        return Some(text);
    }
    js_sys::JSON::stringify(value).ok().and_then(|text| text.as_string())
}

/// Logs a message that could not be understood (visible in the browser console).
pub(super) fn warn_unparseable_message(error: &serde_json::Error) {
    web_sys::console::warn_1(&format!("MopiMopi: ignoring a message from ACT that could not be parsed: {error}").into());
}
