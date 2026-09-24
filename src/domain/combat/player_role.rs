//! The role that decides which table filter and colour a player uses.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayerRole {
    Tank,
    Healer,
    #[default]
    Damage,
    Crafter,
    Gatherer,
    /// A named companion or an unknown NPC that has an owner (for example a chocobo).
    OwnedCombatant,
}

impl PlayerRole {
    /// The key used for this role in the colour palette and in the original's data files.
    pub fn palette_key(self) -> &'static str {
        match self {
            PlayerRole::Tank => "Tanker",
            PlayerRole::Healer => "Healer",
            PlayerRole::Damage => "DPS",
            PlayerRole::Crafter => "Crafter",
            PlayerRole::Gatherer => "Gathering",
            PlayerRole::OwnedCombatant => "CBO",
        }
    }
}
