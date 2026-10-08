//! `WebsocketUrl`: a WebSocket address checked on construction. A value object: once
//! made it cannot be invalid, it is compared by value, and it never changes.

use std::fmt;
use std::str::FromStr;

/// A WebSocket address. The only way to make one is `parse` (or `FromStr` /
/// `TryFrom`), which accepts only `ws://` and `wss://`, so an unchecked string can
/// never reach `JsWebsocketClient::connect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserWebsocketUrl {
    /// `ws://`: unencrypted, as `OverlayPlugin` serves it on this machine.
    Ws(String),
    /// `wss://`: encrypted.
    Wss(String),
}

impl BrowserWebsocketUrl {
    pub fn parse(text: &str) -> Result<Self, UrlError> {
        let (url, rest) = if let Some(rest) = text.strip_prefix("wss://") {
            (BrowserWebsocketUrl::Wss(text.to_owned()), rest)
        } else if let Some(rest) = text.strip_prefix("ws://") {
            (BrowserWebsocketUrl::Ws(text.to_owned()), rest)
        } else {
            return Err(UrlError::UnsupportedScheme(text.to_owned()));
        };
        if rest.is_empty() {
            return Err(UrlError::MissingHost(text.to_owned()));
        }
        Ok(url)
    }

    pub fn as_str(&self) -> &str {
        match self {
            BrowserWebsocketUrl::Ws(url) | BrowserWebsocketUrl::Wss(url) => url,
        }
    }

    /// True for `wss://`. Nothing reads this yet (the app has no automatic ws-vs-wss handling),
    /// but it is real, tested behaviour (see this file's tests), kept as API for that later.
    #[allow(dead_code)]
    pub fn is_secure(&self) -> bool {
        matches!(self, BrowserWebsocketUrl::Wss(_))
    }
}

impl FromStr for BrowserWebsocketUrl {
    type Err = UrlError;
    fn from_str(text: &str) -> Result<Self, UrlError> {
        BrowserWebsocketUrl::parse(text)
    }
}

impl TryFrom<&str> for BrowserWebsocketUrl {
    type Error = UrlError;
    fn try_from(text: &str) -> Result<Self, UrlError> {
        BrowserWebsocketUrl::parse(text)
    }
}

impl TryFrom<String> for BrowserWebsocketUrl {
    type Error = UrlError;
    fn try_from(text: String) -> Result<Self, UrlError> {
        BrowserWebsocketUrl::parse(&text)
    }
}

impl fmt::Display for BrowserWebsocketUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a URL was not accepted. Each case keeps the offending text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlError {
    /// Does not start with `ws://` or `wss://` (checked case-sensitively).
    UnsupportedScheme(String),
    /// Nothing follows the scheme, as in `"ws://"`.
    MissingHost(String),
    /// Scheme accepted, but the browser refused the address (for example, a space
    /// in the host name). Only the browser can tell; reported by `connect`.
    RejectedByBrowser { url: String, detail: String },
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnsupportedScheme(url) => write!(f, "{url:?} must start with ws:// or wss://"),
            Self::MissingHost(url) => write!(f, "{url:?} has no address after the scheme"),
            Self::RejectedByBrowser { url, detail } => write!(f, "the browser refused {url:?}: {detail}"),
        }
    }
}

impl std::error::Error for UrlError {}

#[cfg(test)]
mod tests {
    use super::{UrlError, BrowserWebsocketUrl};

    #[test]
    fn accepts_ws_and_wss() {
        assert_eq!(BrowserWebsocketUrl::parse("ws://127.0.0.1:10501/ws"), Ok(BrowserWebsocketUrl::Ws("ws://127.0.0.1:10501/ws".into())));
        assert_eq!(BrowserWebsocketUrl::parse("wss://example.com"), Ok(BrowserWebsocketUrl::Wss("wss://example.com".into())));
        assert!(BrowserWebsocketUrl::parse("wss://example.com").unwrap().is_secure());
        assert!(!BrowserWebsocketUrl::parse("ws://example.com").unwrap().is_secure());
    }

    #[test]
    fn rejects_other_schemes() {
        for text in ["http://example.com", "https://example.com", "127.0.0.1:10501", "", "not a url", "ws:/x", "ws:x"] {
            assert_eq!(BrowserWebsocketUrl::parse(text), Err(UrlError::UnsupportedScheme(text.into())), "{text:?}");
        }
    }

    #[test]
    fn scheme_check_is_case_sensitive_and_exact() {
        // exactly starts_with("ws://") || starts_with("wss://"): no case folding, no trimming
        for text in ["WS://example.com", "Wss://example.com", " ws://example.com"] {
            assert_eq!(BrowserWebsocketUrl::parse(text), Err(UrlError::UnsupportedScheme(text.into())), "{text:?}");
        }
    }

    #[test]
    fn rejects_a_scheme_with_nothing_after_it() {
        assert_eq!(BrowserWebsocketUrl::parse("ws://"), Err(UrlError::MissingHost("ws://".into())));
        assert_eq!(BrowserWebsocketUrl::parse("wss://"), Err(UrlError::MissingHost("wss://".into())));
    }

    #[test]
    fn parses_through_standard_traits() {
        let from_str: BrowserWebsocketUrl = "ws://localhost".parse().unwrap();
        let try_from = BrowserWebsocketUrl::try_from(String::from("ws://localhost")).unwrap();
        assert_eq!(from_str, try_from);
        assert_eq!(from_str.to_string(), "ws://localhost");
    }
}