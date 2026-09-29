//! Small pieces used by several screens.
//!
//! * `color_conversion`  – hex colours to `rgba(...)`
//! * `style_values`      – reads settings in the units inline `style` variables need
//! * `palette`          – graph bar colours
//! * `row_identity`     – element ids of table rows
//! * `text_display`     – text fragments and job icons as DOM
//! * `rankings_source`  – which rankings a table draws (live or the settings preview sample)
//! * `switch_and_icon`  – on/off switch and row icons
//! * `option_choice`    – settings values as list keys
//! * `safe_markup`      – renders the HTML fragments of the translation files without `innerHTML`

pub mod color_conversion;
pub mod option_choice;
pub mod palette;
pub mod rankings_source;
pub mod row_identity;
pub mod safe_markup;
pub mod style_values;
pub mod switch_and_icon;
pub mod text_display;
