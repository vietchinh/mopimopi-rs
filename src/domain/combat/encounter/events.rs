//! Domain events: facts that happened in the fight, named in the past tense.
//! `ActConnected`/`ActDisconnected` are not here: they are about the connection, an application concern.

use crate::domain::combat::EncounterKey;

/// An encounter that was running has ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncounterEnded {
    pub key: EncounterKey,
}

/// ACT told us who "you" are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YouChanged {
    pub name: String,
}
