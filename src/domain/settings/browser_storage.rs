//! `localStorage` access. Failures (private mode, quota) are ignored: settings then simply
//! are not remembered.

pub fn read_local_storage(key: &str) -> Option<String> {
    web_sys::window()?.local_storage().ok()??.get_item(key).ok()?
}

pub fn write_local_storage(key: &str, value: &str) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|window| window.local_storage()) {
        let _ = storage.set_item(key, value);
    }
}
