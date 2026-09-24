//! First-run language: the overlay UI language that matches the browser language.

/// Returns one of the overlay's language codes: KR, JP, CN, DE, FR or EN.
pub(super) fn detect_language_code() -> &'static str {
    let browser_language = web_sys::window()
        .and_then(|window| window.navigator().language())
        .unwrap_or_default()
        .to_lowercase();
    language_code_for_browser_language(&browser_language)
}

fn language_code_for_browser_language(browser_language: &str) -> &'static str {
    const PREFIX_TO_CODE: [(&str, &str); 5] = [("ko", "KR"), ("ja", "JP"), ("zh", "CN"), ("de", "DE"), ("fr", "FR")];
    PREFIX_TO_CODE
        .iter()
        .find(|(prefix, _)| browser_language.starts_with(prefix))
        .map(|(_, code)| *code)
        .unwrap_or("EN")
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/language_detection.rs"]
mod tests;
