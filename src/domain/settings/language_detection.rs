//! First-run language: the overlay UI language that matches the browser language.

/// Maps a lower-case browser language (e.g. "ko-kr") to one of the overlay's language codes: KR, JP, CN, DE, FR or EN.
pub fn language_code_for_browser_language(browser_language: &str) -> &'static str {
    const PREFIX_TO_CODE: [(&str, &str); 5] = [("ko", "KR"), ("ja", "JP"), ("zh", "CN"), ("de", "DE"), ("fr", "FR")];
    PREFIX_TO_CODE
        .iter()
        .find(|(prefix, _)| browser_language.starts_with(prefix))
        .map_or("EN", |(_, code)| *code)
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/language_detection.rs"]
mod tests;
