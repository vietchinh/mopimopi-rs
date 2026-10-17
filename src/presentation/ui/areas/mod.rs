//! The settings, one typed struct per area of the UI.
//!
//! A component reads only the area it draws, from `SettingsView` (provided once, by `App`): there are no settings keys in the components,
//! and a component is only drawn again when its own area changes, not for every setting. Each area struct is the only Rust code that
//! knows its area's settings keys (`from_raw`) and the names of the CSS variables its area reads (`*_vars`), the CSS side of which is the
//! contract at the top of the area's file in `css/`.
//!
//! * `nav`     – the navigation bar of the live screen, the settings preview and the history screen
//! * `table`   – the damage and healing tables: order, rows, jobs, and the style of the header and the body
//! * `columns` – the columns of those tables
//! * `bars`    – the graph bars behind the rows
//! * `raid`    – raid mode, the grid of small cards
//! * `page`    – the language, the page's font size, background and accent colour, and the switches that are not about one area

mod bars;
mod columns;
mod nav;
mod page;
mod raid;
mod table;

pub use bars::{BarSettings, Side};
pub use columns::{Column, ColumnSettings};
pub use nav::{NavSettings, SummaryParts};
pub use page::PageSettings;
pub use raid::RaidSettings;
pub use table::{JobFilter, TableSettings};

use crate::domain::settings::Settings;
use dioxus::prelude::*;

/// Each area as a memo: it only changes when that area's settings do.
#[derive(Clone, Copy)]
pub struct SettingsView {
    pub nav: Memo<NavSettings>,
    pub table: Memo<TableSettings>,
    pub columns: Memo<ColumnSettings>,
    pub bars: Memo<BarSettings>,
    pub raid: Memo<RaidSettings>,
    pub page: Memo<PageSettings>,
}

/// Creates the areas from the settings and provides them to everything below. Call once, from `App`.
pub fn provide_settings_view(settings: Signal<Settings>) -> SettingsView {
    let nav = use_memo(move || NavSettings::from_raw(settings.read().file()));
    let table = use_memo(move || TableSettings::from_raw(settings.read().file()));
    let columns = use_memo(move || ColumnSettings::from_raw(settings.read().file()));
    let bars = use_memo(move || BarSettings::from_raw(settings.read().file()));
    let raid = use_memo(move || RaidSettings::from_raw(settings.read().file()));
    let page = use_memo(move || PageSettings::from_raw(settings.read().file()));
    use_context_provider(|| SettingsView { nav, table, columns, bars, raid, page })
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/areas/contract.rs"]
mod contract;
