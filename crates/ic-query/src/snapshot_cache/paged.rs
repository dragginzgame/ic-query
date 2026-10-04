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

#[cfg(feature = "sns-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletePagedCollection<Row> {
    pub rows: Vec<Row>,
    pub page_count: u32,
    pub last_cursor: Option<String>,
}

///
/// PagedCollectionPage
///
/// Per-page progress and SNS exhaustion evidence for the refresh runner.
///

pub struct PagedCollectionPage {
    #[cfg(feature = "sns-host")]
    page_len: usize,
    new_rows: usize,
    pub last_cursor_text: Option<String>,
}

///
/// PagedCollectionState
///
/// Accumulates validated rows while walking a cursor-based collection.
///

#[cfg(feature = "sns-host")]
pub struct PagedCollectionState<Row, Cursor> {
    rows: Vec<Row>,
    page_count: u32,
    next_cursor: Option<Cursor>,
}

#[cfg(feature = "sns-host")]
impl<Row, Cursor> Default for PagedCollectionState<Row, Cursor> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            page_count: 0,
            next_cursor: None,
        }
    }
}

#[cfg(feature = "sns-host")]
impl<Row, Cursor> PagedCollectionState<Row, Cursor> {
    pub fn new() -> Self {
        Self::default()
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

    pub const fn has_next_cursor(&self) -> bool {
        self.next_cursor.is_some()
    }

    /// Record admitted rows while retaining the unmodified API page length for exhaustion.
    pub fn ingest_page(
        &mut self,
        rows: Vec<Row>,
        page_len: usize,
        next_cursor: Option<Cursor>,
        cursor_text: impl FnOnce(&Cursor) -> String,
    ) -> PagedCollectionPage {
        self.page_count = self.page_count.saturating_add(1);
        let last_cursor_text = next_cursor.as_ref().map(cursor_text);
        let new_rows = rows.len();
        self.rows.extend(rows);
        self.next_cursor = next_cursor;

        PagedCollectionPage {
            page_len,
            new_rows,
            last_cursor_text,
        }
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

impl PagedCollectionPage {
    /// Describe one validated NNS page, whose rows are all newly admitted.
    #[must_use]
    #[cfg(feature = "nns-host")]
    pub const fn new(new_rows: usize, last_cursor_text: Option<String>) -> Self {
        Self {
            #[cfg(feature = "sns-host")]
            page_len: new_rows,
            new_rows,
            last_cursor_text,
        }
    }

    #[cfg(feature = "sns-host")]
    pub fn exhausts_collection(&self, page_size: u32, has_next_cursor: bool) -> bool {
        self.page_len < usize::try_from(page_size).unwrap_or(usize::MAX) || !has_next_cursor
    }

    pub const fn has_no_new_rows(&self) -> bool {
        self.new_rows == 0
    }
}
