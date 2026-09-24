//! Every WebSocket / callback message shape ACT and OverlayPlugin can send, as serde types.
//!
//! * OverlayPlugin:  `{"type":"CombatData", "Encounter":..., "Combatant":..., "isActive":...}`
//!                   `{"type":"ChangePrimaryPlayer", "charName":"..."}`
//! * MiniParse:      `{"type":"broadcast", "msgtype":"CombatData", "msg":{...}}`
//!                   `{"type":"broadcast", "msgtype":"SendCharName", "msg":{"charName":"..."}}`
//! * Anything else (keep-alives, subscription replies) is parsed as `Ignored`.

use super::combat_data_message::CombatDataMessage;
use serde::Deserialize;

/// What the rest of the program cares about, whichever protocol delivered it.
#[derive(Debug, PartialEq)]
pub enum ActEvent {
    CombatData(CombatDataMessage),
    LocalPlayerName(String),
}

/// Only the two keys that say what kind of message this is. Everything else is skipped by the
/// parser without being stored, which keeps the first pass over a large message cheap.
#[derive(Deserialize)]
struct MessageKind {
    #[serde(default, rename = "type")]
    message_type: Option<String>,
    /// Present on MiniParse broadcasts: what the broadcast carries.
    #[serde(default, rename = "msgtype")]
    broadcast_type: Option<String>,
}

#[derive(Deserialize)]
struct CharacterNamePayload {
    #[serde(rename = "charName")]
    character_name: String,
}

/// MiniParse `broadcast` carrying combat data: the data sits under `msg`.
#[derive(Deserialize)]
struct BroadcastCombatData {
    #[serde(rename = "msg")]
    combat_data: CombatDataMessage,
}

/// MiniParse `broadcast` carrying the local character's name.
#[derive(Deserialize)]
struct BroadcastCharacterName {
    #[serde(rename = "msg")]
    payload: CharacterNamePayload,
}

/// Parses one message. `Ok(None)` means "valid but not interesting"; `Err` means malformed.
///
/// Two passes: the first reads only the message kind, the second reads the payload straight into
/// its final type. (A tagged enum would copy the whole message into a temporary tree first, which
/// was the slowest part of the program.)
pub fn parse_incoming_message(json_text: &str) -> Result<Option<ActEvent>, serde_json::Error> {
    let kind: MessageKind = serde_json::from_str(json_text)?;
    let event = match (kind.message_type.as_deref(), kind.broadcast_type.as_deref()) {
        (Some("CombatData"), _) => Some(ActEvent::CombatData(serde_json::from_str(json_text)?)),
        (Some("ChangePrimaryPlayer"), _) => {
            let payload: CharacterNamePayload = serde_json::from_str(json_text)?;
            Some(ActEvent::LocalPlayerName(payload.character_name))
        }
        (Some("broadcast"), Some("CombatData")) => {
            let broadcast: BroadcastCombatData = serde_json::from_str(json_text)?;
            Some(ActEvent::CombatData(broadcast.combat_data))
        }
        (Some("broadcast"), Some("SendCharName")) => {
            let broadcast: BroadcastCharacterName = serde_json::from_str(json_text)?;
            Some(ActEvent::LocalPlayerName(broadcast.payload.character_name))
        }
        _ => None,
    };
    Ok(event)
}

/// Parses a bare combat data object (the detail of the legacy `onOverlayDataUpdate` DOM event).
pub fn parse_bare_combat_data(json_text: &str) -> Result<CombatDataMessage, serde_json::Error> {
    serde_json::from_str(json_text)
}

#[cfg(test)]
#[path = "../../../tests/unit/models/act_data/incoming_message.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/unit/models/act_data/incoming_message_capture_tests.rs"]
mod capture_tests;
