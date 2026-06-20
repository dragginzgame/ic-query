use std::collections::HashSet;

///
/// CompletePagedCollection
///
/// Deduplicated rows and pagination metadata from a complete API walk.
///

pub struct CompletePagedCollection<Row> {
    pub rows: Vec<Row>,
    pub page_count: u32,
    pub last_cursor: Option<String>,
}

///
/// PagedCollectionPage
///
/// Per-page counters used to decide whether pagination has completed.
///

pub struct PagedCollectionPage {
    page_len: usize,
    new_rows: usize,
    pub last_cursor_text: Option<String>,
}

///
/// PagedCollectionState
///
/// Accumulates unique rows while walking a cursor-based collection.
///

pub struct PagedCollectionState<Row, Cursor> {
    rows: Vec<Row>,
    seen_row_ids: HashSet<String>,
    page_count: u32,
    next_cursor: Option<Cursor>,
}

impl<Row, Cursor> Default for PagedCollectionState<Row, Cursor> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            seen_row_ids: HashSet::new(),
            page_count: 0,
            next_cursor: None,
        }
    }
}

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

    pub fn ingest_page(
        &mut self,
        rows: Vec<Row>,
        next_cursor: Option<Cursor>,
        cursor_text: impl FnOnce(&Cursor) -> String,
        row_id: impl Fn(&Row) -> String,
    ) -> PagedCollectionPage {
        self.page_count = self.page_count.saturating_add(1);
        let page_len = rows.len();
        let last_cursor_text = next_cursor.as_ref().map(cursor_text);
        let mut new_rows = 0_usize;
        for row in rows {
            if self.seen_row_ids.insert(row_id(&row)) {
                new_rows = new_rows.saturating_add(1);
                self.rows.push(row);
            }
        }
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
    pub fn exhausts_collection(&self, page_size: u32, has_next_cursor: bool) -> bool {
        self.page_len < usize::try_from(page_size).unwrap_or(usize::MAX)
            || !has_next_cursor
            || self.new_rows == 0
    }
}
