//! Every `OverlayPlugin` WebSocket message, as one enum.
//!
//! * `{"type":"CombatData", "Encounter":..., "Combatant":..., "isActive":...}`
//! * `{"type":"ChangePrimaryPlayer", "charID":..., "charName":"..."}`
//! * Any other type, including `MiniParse`'s old `broadcast` messages, is `NotSupported`.

use std::fmt;
use super::combat_data_message::CombatDataMessage;
use serde::{de, Deserialize, Deserializer};
use serde::de::{IgnoredAny, MapAccess, Visitor};
use serde::de::value::MapAccessDeserializer;

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
        match serde_json::from_str::<TagFirst>(json_text) {
            Ok(TagFirst(message)) => Ok(message),
            Err(error) => Err(error),
        }
    }

    #[allow(dead_code)]
    pub fn kind(&self) -> MessageType {
        match self {
            Self::CombatData(_) => MessageType::CombatData,
            Self::ChangePrimaryPlayer(_) => MessageType::ChangePrimaryPlayer,
            Self::NotSupported => MessageType::NotSupported,
        }
    }

    #[allow(dead_code)]
    pub fn combat_data(&self) -> Option<&CombatDataMessage> {
        match self {
            Self::CombatData(combat_data) => Some(combat_data),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn change_primary_player(&self) -> Option<&ChangePrimaryPlayer> {
        match self {
            Self::ChangePrimaryPlayer(player) => Some(player),
            _ => None,
        }
    }
}

pub struct TagFirst(pub OverlayMessage);

impl<'de> Deserialize<'de> for TagFirst {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = TagFirst;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("an OverlayPlugin message with \"type\" first") }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<TagFirst, A::Error> {
                let key: std::borrow::Cow<'de, str> = map.next_key()?.ok_or_else(|| de::Error::missing_field("type"))?;
                if key != "type" { return Err(de::Error::custom("\"type\" is not the first key")); }
                let kind: std::borrow::Cow<'de, str> = map.next_value()?;
                let rest = MapAccessDeserializer::new(map);
                Ok(TagFirst(match &*kind {
                    "CombatData" => OverlayMessage::CombatData(CombatDataMessage::deserialize(rest)?),
                    "ChangePrimaryPlayer" => OverlayMessage::ChangePrimaryPlayer(ChangePrimaryPlayer::deserialize(rest)?),
                    _ => { IgnoredAny::deserialize(rest)?; OverlayMessage::NotSupported }
                }))
            }
        }
        deserializer.deserialize_map(V)
    }
}


#[test]
fn type_not_first_still_parses() {
    let text = r#"{"charID":1,"charName":"Future Fade","type":"ChangePrimaryPlayer"}"#;
    let message = OverlayMessage::parse(text).unwrap();
    assert_eq!(message.change_primary_player().unwrap().char_name, "Future Fade");
}


#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/overlay_message.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/pet_merging.rs"]
mod pet_merging_tests;
