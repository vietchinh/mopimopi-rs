//! The two phases of a fight's life: it is running, then it is over. ACT sends no identity for a fight, only the
//! current state of it in every message, so the Encounter value is a snapshot of one phase; the fight's identity is
//! its `EncounterKey`, which only matters once the fight is over (that is when the history records it).

/// Whether the fight is still going on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    Running,
    /// Also the phase before any message has arrived: nothing is going on.
    #[default]
    Over,
}

impl Phase {
    pub fn of_running(is_running: bool) -> Self {
        if is_running { Phase::Running } else { Phase::Over }
    }

    pub fn is_running(self) -> bool {
        self == Phase::Running
    }
}
