//! Typed styling values: what the settings file says, in the units CSS needs.
//!
//! The settings file stores sizes in tenths of a rem, opacities in percent, colours as hex digits, and a handful of other conventions
//! (a corner is a flag times a radius, a text outline is a shadow of four offsets). Each convention is handled once, here:
//!
//! * `values` – `Size`, `Opacity`, `Paint`, `FontStack`, `Shadow`: each prints as the CSS it stands for, and `Vars`, which writes
//!   `--name:value;` declarations
//! * `reads`  – `StyleReads`, the typed readers on `SettingsFile`: one per convention, each checking in debug builds that the key exists
//! * `shapes` – `TextStyle`, `Line` and `Corners`: shapes several areas have, each knowing the variable names it writes
//!
//! The areas (`ui/areas`) are built from these; each is tested against what the string-based code gave, for each settings profile in
//! `tests/fixtures/settings`.

mod reads;
mod shapes;
mod values;

pub use reads::StyleReads;
pub use shapes::{Corners, Line, TextStyle};
pub use values::{FontStack, Opacity, Paint, Shadow, Size, Vars};

#[cfg(test)]
#[path = "../../../../../tests/unit/presentation/ui/shared/style.rs"]
mod tests;
