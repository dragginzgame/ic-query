//! Module: sns::report::lookup::model
//!
//! Responsibility: resolved SNS lookup model.
//! Does not own: lookup parsing, live source fetching, or report assembly.
//! Boundary: carries one resolved SNS identity and fetch context to builders.

use crate::sns::report::source::{MainnetSns, MainnetSnsList, SnsFetchRequest};

///
/// SnsLookup
///
/// Resolved deployed SNS lookup with its source list and fetch request.
///

pub(in crate::sns::report) struct SnsLookup {
    pub(in crate::sns::report) fetch_request: SnsFetchRequest,
    pub(in crate::sns::report) list: MainnetSnsList,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) sns: MainnetSns,
}
