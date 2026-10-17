//! The History screen: the finished encounters and which one is shown.

use crate::domain::combat::{EncounterKey, History};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct HistoryContext {
    pub encounter_history: Signal<History>,
    /// History entry currently displayed, if any.
    pub viewed_history_key: Signal<Option<EncounterKey>>,
}

impl HistoryContext {
    /// A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let context = Self {
            encounter_history: use_signal(History::default),
            viewed_history_key: use_signal(|| None::<EncounterKey>),
        };
        use_context_provider(|| context)
    }
}
