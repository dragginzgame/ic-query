//! Module: snapshot_cache::paged
//!
//! Responsibility: accumulate admitted rows across paged API walks.
//! Does not own: source fetching, progress rendering, or cache publication.
//! Boundary: tracks page counters and cursors after family-owned page validation.

///
/// CompletePagedCollection
///
/// Admitted rows and pagination metadata from a complete API walk.
///

#[cfg(any(feature = "icrc-host", feature = "sns-host"))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletePagedCollection<Row> {
    pub rows: Vec<Row>,
    pub page_count: u32,
    pub last_cursor: Option<String>,
}

///
/// PagedCollectionPage
///
/// Per-page progress and family-admitted exhaustion evidence for the refresh runner.
///

#[cfg(any(feature = "nns-host", feature = "sns-host"))]
pub struct PagedCollectionPage {
    new_rows: usize,
    pub last_cursor_text: Option<String>,
    pub collection_exhausted: bool,
}

///
/// PagedCollectionState
///
/// Accumulates validated rows while walking a cursor-based collection.
///

#[cfg(any(feature = "icrc-host", feature = "sns-host"))]
pub struct PagedCollectionState<Row, Cursor> {
    rows: Vec<Row>,
    page_count: u32,
    next_cursor: Option<Cursor>,
}

#[cfg(any(feature = "icrc-host", feature = "sns-host"))]
impl<Row, Cursor> Default for PagedCollectionState<Row, Cursor> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(feature = "icrc-host", feature = "sns-host"))]
impl<Row, Cursor> PagedCollectionState<Row, Cursor> {
    pub const fn new() -> Self {
        Self {
            rows: Vec::new(),
            page_count: 0,
            next_cursor: None,
        }
    }

    pub const fn page_count(&self) -> u32 {
        self.page_count
    }

    pub const fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub const fn next_cursor(&self) -> Option<&Cursor> {
        self.next_cursor.as_ref()
    }

    /// Record rows and a cursor only after family-owned page validation.
    pub fn ingest(&mut self, rows: Vec<Row>, next_cursor: Option<Cursor>) {
        self.rows.extend(rows);
        self.page_count = self.page_count.saturating_add(1);
        self.next_cursor = next_cursor;
    }

    pub fn into_complete(
        self,
        cursor_text: impl FnOnce(&Cursor) -> String,
    ) -> CompletePagedCollection<Row> {
        CompletePagedCollection {
            rows: self.rows,
            page_count: self.page_count,
            last_cursor: self.next_cursor.as_ref().map(cursor_text),
        }
    }
}

#[cfg(any(feature = "nns-host", feature = "sns-host"))]
impl PagedCollectionPage {
    /// Describe one validated page and its family-owned completion decision.
    #[must_use]
    pub const fn new(
        new_rows: usize,
        last_cursor_text: Option<String>,
        collection_exhausted: bool,
    ) -> Self {
        Self {
            new_rows,
            last_cursor_text,
            collection_exhausted,
        }
    }

    pub const fn has_no_new_rows(&self) -> bool {
        self.new_rows == 0
    }
}
