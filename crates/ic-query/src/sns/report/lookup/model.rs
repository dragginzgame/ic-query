//! Module: sns::report::lookup::model
//!
//! Responsibility: resolved SNS lookup model.
//! Does not own: lookup parsing, live source fetching, or report assembly.
//! Boundary: carries one resolved SNS identity and fetch context to builders.

use crate::report::CollectionSourceRequest;
use crate::sns::report::source::MainnetSns;

///
/// SnsLookup
///
/// Resolved deployed SNS lookup with targeted discovery provenance and its fetch request.
///

pub(in crate::sns::report) struct SnsLookup {
    pub(in crate::sns::report) fetch_request: CollectionSourceRequest,
    pub(in crate::sns::report) sns: MainnetSns,
}
