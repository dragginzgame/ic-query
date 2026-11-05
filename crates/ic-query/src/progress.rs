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
        /// Collection identity and admitted counters, without presentation text.
        progress: PagedQueryProgress,
        /// Current lifecycle state for the progress message.
        state: QueryProgressState,
    },
}

///
/// QueryProgressCollection
///
/// Collection whose paging counters are being reported.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryProgressCollection {
    /// Public NNS neuron index.
    NnsNeurons,
    /// NNS Governance proposals.
    NnsProposals,
    /// Complete SNS neuron collection.
    SnsNeurons,
    /// Complete ledger/index account transaction history.
    IcrcAccountTransactions,
}

///
/// PagedQueryProgress
///
/// Admitted page and row counts with optional human target identity.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PagedQueryProgress {
    /// Collection whose counters were admitted by its collector.
    pub collection: QueryProgressCollection,
    /// Optional observed target name, escaped by the presentation owner.
    pub target: Option<String>,
    /// Number of admitted source pages.
    pub pages_fetched: u32,
    /// Number of admitted retained rows.
    pub rows_fetched: usize,
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

pub trait QueryProgress: Send {
    /// Receive one progress event.
    fn report(&mut self, event: QueryProgressEvent);
}

impl<Reporter> QueryProgress for Reporter
where
    Reporter: FnMut(QueryProgressEvent) + Send,
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

#[cfg(any(
    feature = "icrc-host",
    feature = "nns-host",
    feature = "subnet-catalog-host",
    test
))]
pub struct IgnoreQueryProgress;

#[cfg(any(
    feature = "icrc-host",
    feature = "nns-host",
    feature = "subnet-catalog-host",
    test
))]
impl QueryProgress for IgnoreQueryProgress {
    fn report(&mut self, _event: QueryProgressEvent) {}
}
