//! Every `OverlayPlugin` WebSocket message, as one enum.
//!
//! * `{"type":"CombatData", "Encounter":..., "Combatant":..., "isActive":...}`
//! * `{"type":"ChangePrimaryPlayer", "charID":..., "charName":"..."}`
//! * Any other type, including `MiniParse`'s old `broadcast` messages, is `NotSupported`.

use super::combat_data_message::CombatDataMessage;
use serde::de::value::MapAccessDeserializer;
use serde::de::{Error as _, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::infrastructure::act) enum OverlayMessage {
    CombatData(CombatDataMessage),
    ChangePrimaryPlayer(ChangePrimaryPlayer),
    /// Any other `"type"` (`ChangeZone`, `LogLine`, `MiniParse` `broadcast`, ...). Its fields are discarded.
    NotSupported,
}

/// What a first look at a message found. `OverlayPlugin` writes `"type"` first, and then the message is read once, straight into
/// its data. A derived internally tagged enum would instead copy the whole message into a buffer to find the tag and then read it
/// a second time, which doubled the cost of every update. A message with its `"type"` elsewhere is skipped through once to find it
/// (cheap: nothing is kept) and then read by `parse` as the kind it turned out to be.
enum FirstLook {
    Done(OverlayMessage),
    TypeNotFirst(Option<String>),
}

impl<'de> Deserialize<'de> for FirstLook {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LookVisitor;

        impl<'de> Visitor<'de> for LookVisitor {
            type Value = FirstLook;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an OverlayPlugin message: an object with a \"type\"")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<FirstLook, A::Error> {
                let Some(first_key) = map.next_key::<String>()? else { return Err(A::Error::missing_field("type")) };
                if first_key != "type" {
                    map.next_value::<IgnoredAny>()?;
                    let mut kind = None;
                    while let Some(key) = map.next_key::<String>()? {
                        if key == "type" {
                            kind = Some(map.next_value::<String>()?);
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    return Ok(FirstLook::TypeNotFirst(kind));
                }
                let kind = map.next_value::<String>()?;
                let message = match kind.as_str() {
                    "CombatData" => OverlayMessage::CombatData(CombatDataMessage::deserialize(MapAccessDeserializer::new(map))?),
                    "ChangePrimaryPlayer" => OverlayMessage::ChangePrimaryPlayer(ChangePrimaryPlayer::deserialize(MapAccessDeserializer::new(map))?),
                    _ => {
                        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                        OverlayMessage::NotSupported
                    }
                };
                Ok(FirstLook::Done(message))
            }
        }

        deserializer.deserialize_map(LookVisitor)
    }
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

impl OverlayMessage {
    pub fn parse(json_text: &str) -> Result<Self, serde_json::Error> {
        match serde_json::from_str::<FirstLook>(json_text)? {
            FirstLook::Done(message) => Ok(message),
            FirstLook::TypeNotFirst(Some(kind)) => match kind.as_str() {
                "CombatData" => serde_json::from_str(json_text).map(Self::CombatData),
                "ChangePrimaryPlayer" => serde_json::from_str(json_text).map(Self::ChangePrimaryPlayer),
                _ => Ok(Self::NotSupported),
            },
            FirstLook::TypeNotFirst(None) => Err(serde_json::Error::missing_field("type")),
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

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/overlay_message.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/pet_merging.rs"]
mod pet_merging_tests;
