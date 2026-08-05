//! Module: progress
//!
//! Responsibility: define process-agnostic query progress events.
//! Does not own: terminal detection, stderr output, or command presentation.
//! Boundary: lets host operations report progress without choosing an output sink.

use std::path::PathBuf;

///
/// QueryProgressEvent
///
/// Structured progress emitted by host-backed query and refresh operations.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryProgressEvent {
    /// A cache-backed read is about to refresh from a live endpoint.
    CacheRefresh {
        /// Human-readable cache component name.
        component: String,
        /// Cache path that will receive the validated replacement.
        path: PathBuf,
        /// Explicit live endpoint used to create the cache.
        source_endpoint: String,
    },
    /// Progress from a complete paged snapshot refresh.
    PagedRefresh {
        /// Human-readable progress text owned by the refresh family.
        text: String,
        /// Current lifecycle state for the progress message.
        state: QueryProgressState,
    },
}

///
/// QueryProgressState
///
/// Lifecycle state attached to a paged refresh progress message.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryProgressState {
    /// The refresh is still fetching or persisting pages.
    Running,
    /// The complete snapshot was fetched successfully.
    Complete,
    /// A source or persistence operation failed.
    Failed,
    /// A configured page limit stopped the refresh before completion.
    Stopped,
    /// Paging stopped making forward progress before completion.
    Stalled,
}

///
/// QueryProgress
///
/// Process-agnostic sink for structured host-operation progress.
///

pub trait QueryProgress {
    /// Receive one progress event.
    fn report(&mut self, event: QueryProgressEvent);
}

impl<Reporter> QueryProgress for Reporter
where
    Reporter: FnMut(QueryProgressEvent),
{
    fn report(&mut self, event: QueryProgressEvent) {
        self(event);
    }
}

///
/// IgnoreQueryProgress
///
/// No-op progress sink used by silent library entry points.
///

#[cfg(any(feature = "host", feature = "icrc-host", feature = "sns-host", test))]
pub struct IgnoreQueryProgress;

#[cfg(any(feature = "host", feature = "icrc-host", feature = "sns-host", test))]
impl QueryProgress for IgnoreQueryProgress {
    fn report(&mut self, _event: QueryProgressEvent) {}
}
