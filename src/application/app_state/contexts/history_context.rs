//! The History screen: the finished encounters and which one is shown.

use crate::application::app_state::encounter_history::HistoryEntry;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct HistoryContext {
    pub encounter_history: Signal<Vec<HistoryEntry>>,
    /// Number of encounters recorded in a row in the same zone.
    pub encounters_in_current_zone: Signal<usize>,
    /// History entry currently displayed, if any.
    pub viewed_history_key: Signal<Option<String>>,
}

impl HistoryContext {
    /// A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let context = Self {
            encounter_history: use_signal(Vec::new),
            encounters_in_current_zone: use_signal(|| 0usize),
            viewed_history_key: use_signal(|| None::<String>),
        };
        use_context_provider(|| context)
    }
}
