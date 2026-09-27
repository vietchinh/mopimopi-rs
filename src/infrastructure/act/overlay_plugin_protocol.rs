//! OverlayPlugin's protocol, as pure functions and types: the address, the subscribe
//! message, and what each incoming message means. No I/O: everything here is tested
//! directly. The connection itself is wired up by the UI (see
//! `ui::overlay_plugin_provider`), on top of `browser_websocket::JsWebsocketClient`.

use crate::infrastructure::browser_websocket::{BrowserWebsocketUrl, UrlError};
use std::fmt;
use crate::infrastructure::act::data::{CombatDataMessage, OverlayMessage};
use crate::infrastructure::act::overlay_plugin_protocol::OverlayPluginEvent::{CombatData, PrimaryPlayerChanged};
// ---------- the URL ----------

/// OverlayPlugin's WebSocket address: a `ws://` or `wss://` URL ending in `/ws`,
/// such as `ws://127.0.0.1:10501/ws`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayPluginUrl(BrowserWebsocketUrl);

impl OverlayPluginUrl {
    pub fn parse(text: &str) -> Result<Self, OverlayPluginUrlError> {
        let url = BrowserWebsocketUrl::parse(text)?;
        if !url.as_str().ends_with("/ws") {
            return Err(OverlayPluginUrlError::MissingWsPath(text.to_owned()));
        }
        Ok(Self(url))
    }

    pub fn websocket_url(&self) -> &BrowserWebsocketUrl {
        &self.0
    }
}

impl fmt::Display for OverlayPluginUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayPluginUrlError {
    /// Not a valid WebSocket URL; see `UrlError`.
    Websocket(UrlError),
    /// A valid WebSocket URL, but not ending in `/ws`.
    MissingWsPath(String),
}

impl From<UrlError> for OverlayPluginUrlError {
    fn from(error: UrlError) -> Self {
        Self::Websocket(error)
    }
}

impl fmt::Display for OverlayPluginUrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Websocket(error) => error.fmt(f),
            Self::MissingWsPath(url) => write!(f, "{url:?} must end with /ws (OverlayPlugin's WebSocket path)"),
        }
    }
}

impl std::error::Error for OverlayPluginUrlError {}

// ---------- the subscribe message ----------

/// OverlayPlugin sends nothing until subscribed, and forgets the subscription with the
/// connection, so this is sent on every (re)open. The format common.js uses.
pub fn subscribe_message() -> String {
    r#"{"call":"subscribe","events":["CombatData","ChangePrimaryPlayer"]}"#.to_string()
}

// ---------- incoming messages ----------

/// What an OverlayPlugin message means, in the domain's terms.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayPluginEvent {
    /// A `CombatData` update.
    CombatData(CombatDataMessage),
    /// `ChangePrimaryPlayer`: the local character, sent after every (re)subscribe.
    PrimaryPlayerChanged(String),
}

/// What one message means. Pure: no I/O, so it is tested directly.
/// `Ok(None)` for valid messages that are not events (for example, replies to calls).
pub fn interpret(text: &str) -> Result<Option<OverlayPluginEvent>, serde_json::Error> {
    match OverlayMessage::parse(text)? {
        OverlayMessage::CombatData(combat_data) => {
            Ok(Some(CombatData(combat_data)))
        }
        OverlayMessage::ChangePrimaryPlayer(player) => {
            Ok(Some(PrimaryPlayerChanged(player.char_name)))
        }
        OverlayMessage::NotSupported => Ok(None),
    }
}

// ---------- the client ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_must_end_with_ws() {
        assert!(OverlayPluginUrl::parse("ws://127.0.0.1:10501/ws").is_ok());
        assert!(OverlayPluginUrl::parse("wss://example.com/ws").is_ok());
        for text in ["ws://127.0.0.1:10501", "ws://127.0.0.1:10501/", "ws://127.0.0.1:10501/ws/", "ws://127.0.0.1:10501/MiniParse"] {
            assert_eq!(OverlayPluginUrl::parse(text), Err(OverlayPluginUrlError::MissingWsPath(text.into())), "{text:?}");
        }
    }

    #[test]
    fn url_scheme_errors_are_passed_through() {
        assert_eq!(
            OverlayPluginUrl::parse("http://127.0.0.1:10501/ws"),
            Err(OverlayPluginUrlError::Websocket(UrlError::UnsupportedScheme("http://127.0.0.1:10501/ws".into())))
        );
    }

    #[test]
    fn subscribes_to_the_events_we_handle() {
        let message = subscribe_message();
        assert!(message.contains(r#""call":"subscribe""#));
        assert!(message.contains("CombatData") && message.contains("ChangePrimaryPlayer"));
    }

    #[test]
    fn interprets_combat_data() {
        let text = r#"{"type":"CombatData","Encounter":{"title":"Striking Dummy","DURATION":"10","damage":"24,466","CurrentZoneName":"Shirogane"},"Combatant":{"YOU":{"name":"YOU","Job":"Drk","damage":"24466","dps":"2740.98","maxhit":"attack-8273","MAXHIT":"8273"}},"isActive":"true"}"#;
        let Ok(Some(OverlayPluginEvent::CombatData(e))) = interpret(text) else { panic!("not combat data") };
        assert_eq!((e.encounter.title.as_str(), e.encounter.zone_name.as_str(), e.encounter.total_damage), ("Striking Dummy", "Shirogane", 24466.0));
        assert_eq!(e.combatants[0].name, "YOU");
        assert_eq!(e.combatants[0].damage_per_second, 2740.98);
    }

    #[test]
    fn interprets_the_primary_player() {
        let event = interpret(r#"{"type":"ChangePrimaryPlayer","charID":268938170,"charName":"Future Fade"}"#).unwrap();
        assert!(matches!(event, Some(OverlayPluginEvent::PrimaryPlayerChanged(ref name)) if name.as_str() == "Future Fade"));
    }

    #[test]
    fn ignores_other_messages_and_reports_malformed_ones() {
        assert!(matches!(interpret(r#"{"type":"ChangeZone","zoneID":132}"#), Ok(None)));
        assert!(matches!(interpret(r#"{"rseq":1,"data":[]}"#), Ok(None)));
        assert!(interpret("{not json").is_err());
    }
}