//! The pop-up menu that is currently open.

use dioxus::prelude::*;
use serde_json::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum Dropdown {
    /// The ⋮ menu; its entries depend on the current screen.
    Navigation,
    /// Pick one value of a setting: `setting_key` and the schema's value -> label object.
    ChooseOption { setting_key: String, choices: Value },
    /// Several on/off settings as (setting key, label) pairs.
    ToggleOptions { options: Vec<(String, String)> },
    /// Left / center / right for a table column (`field` is "alignHeader" or "alignBody").
    ChooseColumnAlignment { column: String, field: String },
}

#[derive(Clone, Copy)]
pub struct DropdownContext {
    pub open_dropdown: Signal<Option<Dropdown>>,
}

impl DropdownContext {
    /// A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let open_dropdown = use_signal(|| None::<Dropdown>);
        use_context_provider(|| Self { open_dropdown })
    }
}
