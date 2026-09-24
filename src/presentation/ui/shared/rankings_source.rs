//! Which data the tables and the top bar draw.

use crate::application::app_state::AppContext;
use dioxus::prelude::*;
use crate::domain::combat::EncounterRankings;
use std::rc::Rc;

/// The live rankings, or the built-in sample fight for the previews on settings pages.
pub fn rankings_to_draw(context: AppContext, is_settings_preview: bool) -> Option<Rc<EncounterRankings>> {
    if is_settings_preview {
        Some(context.sample_rankings.read().clone()) // clones the Rc, not the data
    } else {
        context.rankings.read().clone()
    }
}
