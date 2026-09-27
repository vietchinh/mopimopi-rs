//! A look at a raw `CombatData` message *without parsing or copying it*: is the encounter
//! active, and does it have any combatants?
//!
//! Between fights OverlayPlugin keeps sending the same message every second, e.g. after
//! leaving a trial: `"Combatant":{}` with `"isActive":"true"`. In a WebAssembly GUI the text
//! arrives as a JavaScript string; peeking at it through `JsString` runs the search inside
//! the browser, so a message that is going to be skipped is never copied into WebAssembly
//! memory at all.

const NO_COMBATANTS: &str = r#""Combatant":{},"#;
const ACTIVE_END: &str = r#","isActive":"true"}"#;
const INACTIVE_END: &str = r#","isActive":"false"}"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatDataPeek {
    pub is_active: bool,
    pub has_combatants: bool,
}

pub trait SearchableText {
    fn ends_with(&self, suffix: &str) -> bool;
    fn contains(&self, needle: &str) -> bool;
}

pub fn peek_combat_data<Text: SearchableText + ?Sized>(message: &Text) -> Option<CombatDataPeek> {
    let is_active = match (message.ends_with(ACTIVE_END), message.ends_with(INACTIVE_END)) {
        (true, _) => true,
        (_, true) => false,
        _ => return None,
    };
    let has_combatants = !message.contains(NO_COMBATANTS);
    Some(CombatDataPeek { is_active, has_combatants })
}

impl SearchableText for str {
    fn ends_with(&self, suffix: &str) -> bool {
        str::ends_with(self, suffix)
    }
    fn contains(&self, needle: &str) -> bool {
        str::contains(self, needle)
    }
}

/// Searches inside the browser: the message is not copied into WebAssembly memory.
#[cfg(target_arch = "wasm32")]
impl SearchableText for js_sys::JsString {
    fn ends_with(&self, suffix: &str) -> bool {
        js_sys::JsString::ends_with(self, suffix, self.length() as i32)
    }
    fn contains(&self, needle: &str) -> bool {
        js_sys::JsString::includes(self, needle, 0)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/combat_data_peek.rs"]
mod tests;
