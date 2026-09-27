//! Every `OverlayPlugin` WebSocket message, as one enum.
//!
//! * `{"type":"CombatData", "Encounter":..., "Combatant":..., "isActive":...}`
//! * `{"type":"ChangePrimaryPlayer", "charID":..., "charName":"..."}`
//! * Any other type, including `MiniParse`'s old `broadcast` messages, is `NotSupported`.

use super::combat_data_message::CombatDataMessage;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum OverlayMessage {
    CombatData(CombatDataMessage),
    ChangePrimaryPlayer(ChangePrimaryPlayer),
    /// Any other `"type"` (`ChangeZone`, `LogLine`, `MiniParse` `broadcast`, ...). Its fields are discarded.
    #[serde(other)]
    NotSupported,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ChangePrimaryPlayer {
    #[serde(rename = "charID")]
    pub char_id: u32,
    #[serde(rename = "charName")]
    pub char_name: String,
}

/// Just the `"type"`, without the data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MessageType {
    CombatData,
    ChangePrimaryPlayer,
    NotSupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParseOptions<'a> {
    pub merge_pets_into_owner: bool,
    pub local_player_name: Option<&'a str>,
}

impl OverlayMessage {
    pub fn parse(json_text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_text)
    }

    pub fn kind(&self) -> MessageType {
        match self {
            Self::CombatData(_) => MessageType::CombatData,
            Self::ChangePrimaryPlayer(_) => MessageType::ChangePrimaryPlayer,
            Self::NotSupported => MessageType::NotSupported,
        }
    }

    pub fn combat_data(&self) -> Option<&CombatDataMessage> {
        match self {
            Self::CombatData(combat_data) => Some(combat_data),
            _ => None,
        }
    }

    pub fn change_primary_player(&self) -> Option<&ChangePrimaryPlayer> {
        match self {
            Self::ChangePrimaryPlayer(player) => Some(player),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/overlay_message.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/pet_merging.rs"]
mod pet_merging_tests;
