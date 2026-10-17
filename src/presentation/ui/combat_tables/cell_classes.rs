//! The classes the cells of a table share, written once: the standard table and the history list draw the same header and the same lines between
//! cells. Each is a literal string because Tailwind finds a class by reading the source text; each starts with a space, to follow another class.

/// The line to the right of a body cell, all but the last: the vertical line between cells (`--chrome-cell-border`).
pub(crate) const BODY_CELL_BORDER: &str = " not-last:[border-right:var(--chrome-cell-border)]";

/// A header cell: the header's background and height, the rounded corners of the first and of the last (so they follow whichever columns are
/// shown), and the line to the right of all but the last (`--chrome-header-cell-border`).
pub(crate) const HEADER_CELL: &str =
    " chrome-header-cell first:corner-header-left last:corner-header-right not-last:[border-right:var(--chrome-header-cell-border)]";

/// The table that holds the header cells: its margin, and the text rules of the header (`themed-text`).
pub(crate) const HEADER_TABLE: &str = "tableHeader chrome-header-table themed-text";
