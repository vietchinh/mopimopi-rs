//! Preferences that exist only in this port. They are kept apart from the settings file on purpose.
//!
//! The settings file, its `Mopi2_HAERU` storage key and the shareable "Custom UI Data" code must stay exactly what the
//! original overlay reads and writes (this port is a conformist copy). A preference that the original does not know about
//! therefore lives in its own store with its own export and import, and neither side ever touches the other
//! (DDD: anticorruption between the shared published language and the port's own extensions).

use serde::{Deserialize, Serialize};

/// First characters of an exported code, so it can never be mistaken for (or pasted as) a MopiMopi "Custom UI Data" code.
pub const PORT_CODE_PREFIX: &str = "MOPIPORT1:";

/// Colour for NPC rows that look like themselves, when the user has not picked one (hex, no `#`).
pub const DEFAULT_NPC_COLOR: &str = "4DB6AC";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PortPreferences {
    /// Off by default: NPCs are drawn like the original draws them (the Limit Break icon and colour). On: they get
    /// their own icon and colour.
    pub npc_own_look: bool,
    pub npc_color: String,
}

impl Default for PortPreferences {
    fn default() -> Self {
        Self { npc_own_look: false, npc_color: DEFAULT_NPC_COLOR.to_string() }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidPortCode;

impl PortPreferences {
    /// Text that can be saved and pasted back; starts with `PORT_CODE_PREFIX`.
    pub fn to_code(&self) -> String {
        format!("{PORT_CODE_PREFIX}{}", serde_json::to_string(self).unwrap_or_default())
    }

    /// Reads a code made by `to_code`. Anything else (including a MopiMopi shareable code) is refused.
    pub fn from_code(code: &str) -> Result<Self, InvalidPortCode> {
        let json = code.trim().strip_prefix(PORT_CODE_PREFIX).ok_or(InvalidPortCode)?;
        let mut parsed: PortPreferences = serde_json::from_str(json).map_err(|_| InvalidPortCode)?;
        if !is_hex_color(&parsed.npc_color) {
            parsed.npc_color = DEFAULT_NPC_COLOR.to_string();
        }
        Ok(parsed)
    }

    /// The saved text of the store, repaired: a damaged or missing store gives the defaults.
    pub fn from_stored(text: Option<&str>) -> Self {
        text.and_then(|text| serde_json::from_str::<PortPreferences>(text).ok())
            .map(|mut preferences| {
                if !is_hex_color(&preferences.npc_color) {
                    preferences.npc_color = DEFAULT_NPC_COLOR.to_string();
                }
                preferences
            })
            .unwrap_or_default()
    }

    pub fn to_stored(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Six hex digits, as the settings store colours (no `#`).
pub fn is_hex_color(text: &str) -> bool {
    text.len() == 6 && text.chars().all(|character| character.is_ascii_hexdigit())
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/port_preferences.rs"]
mod tests;
