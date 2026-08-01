//! Module: nns::neuron::report
//!
//! Responsibility: build direct NNS Governance neuron reports.
//! Does not own: Dashboard analytics, authenticated neuron management, or process output.
//! Boundary: preserves the public `NeuronInfo` view and its Governance provenance.

#[cfg(feature = "host")]
mod cache;
mod model;
#[cfg(feature = "host")]
mod source;
mod text;

#[cfg(feature = "host")]
use crate::{HostCacheError, nns::NnsGovernanceQueryError, runtime::RuntimeError};
#[cfg(feature = "host")]
use std::path::PathBuf;
#[cfg(feature = "host")]
use thiserror::Error as ThisError;

#[cfg(feature = "host")]
pub use cache::{
    DEFAULT_NNS_NEURON_REFRESH_LOCK_STALE_SECONDS, NnsNeuronCacheStatusReport,
    NnsNeuronCacheSummary, NnsNeuronRefreshReport, build_nns_neuron_cache_status_report,
    build_nns_neuron_info_report_from_cache, build_nns_neuron_list_report_from_cache,
    nns_neuron_cache_path, nns_neuron_refresh_attempt_path, nns_neuron_refresh_lock_path,
    refresh_nns_neuron_cache, refresh_nns_neuron_cache_with_progress,
    refresh_nns_neuron_cache_with_source,
};
pub use model::{
    NnsKnownNeuronData, NnsNeuronBallotRow, NnsNeuronInfoReport, NnsNeuronInfoRequest,
    NnsNeuronListReport, NnsNeuronListRequest, NnsNeuronRow,
};
#[cfg(feature = "host")]
pub use source::{
    NnsNeuronPage, NnsNeuronSource, build_nns_neuron_info_report,
    build_nns_neuron_info_report_with_source, build_nns_neuron_list_report,
    build_nns_neuron_list_report_with_source,
};
#[cfg(feature = "host")]
pub use text::{nns_neuron_cache_status_report_text, nns_neuron_refresh_report_text};
pub use text::{nns_neuron_info_report_text, nns_neuron_list_report_text};

#[cfg(all(test, feature = "host"))]
mod tests;

/// Default replica endpoint used for direct NNS Governance neuron queries.
pub const DEFAULT_NNS_NEURON_SOURCE_ENDPOINT: &str = "https://icp-api.io";

/// Largest page size accepted by the public Governance neuron index.
pub const NNS_NEURON_MAX_PAGE_SIZE: u32 = 300;

#[cfg(feature = "host")]
const NNS_NEURON_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const NNS_NEURON_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const NNS_NEURON_FETCHED_BY: &str = "ic-query";

///
/// NnsNeuronHostError
///
/// Error returned while querying or caching public NNS neuron information.
///

#[cfg(feature = "host")]
#[derive(Debug, ThisError)]
pub enum NnsNeuronHostError {
    /// The requested network is not the supported mainnet identity.
    #[error(
        "`icq nns neuron` supports only the mainnet `ic` network\n\nThe public neuron index is queried from the Internet Computer mainnet Governance canister.\n\nTry:\n  icq --network ic nns neuron list"
    )]
    UnsupportedNetwork {
        /// Rejected network identity.
        network: String,
    },

    /// The requested page size is outside Governance's supported range.
    #[error("invalid NNS neuron page size {page_size}; expected 1..={max_page_size}")]
    InvalidPageSize {
        /// Rejected page size.
        page_size: u32,
        /// Largest page supported by Governance.
        max_page_size: u32,
    },

    /// Shared NNS Governance transport failed.
    #[error(transparent)]
    GovernanceQuery(#[from] NnsGovernanceQueryError),

    /// Governance returned its typed application-level error.
    #[error("NNS Governance rejected the neuron query with code {error_type}: {message}")]
    Governance {
        /// Raw Governance error type.
        error_type: i32,
        /// Governance error message.
        message: String,
    },

    /// Governance has no publicly readable view for the requested neuron id.
    #[error("NNS neuron {neuron_id} was not found")]
    NeuronNotFound {
        /// Requested neuron identifier.
        neuron_id: u64,
    },

    /// Governance returned a neuron row without its required identifier.
    #[error("NNS Governance returned a neuron row without an id")]
    MissingNeuronId,

    /// A source page violated the ordered neuron pagination contract.
    #[error("invalid NNS neuron page: {reason}")]
    InvalidPage {
        /// Page invariant that failed.
        reason: String,
    },

    /// A capped or stalled refresh stopped before proving API exhaustion.
    #[error(
        "NNS neuron refresh stopped after {pages_fetched} pages and {rows_fetched} rows: {reason}"
    )]
    IncompleteRefresh {
        /// Pages retained before the stop.
        pages_fetched: u32,
        /// Rows retained before the stop.
        rows_fetched: usize,
        /// Completion invariant that failed.
        reason: String,
    },

    /// A stored neuron snapshot failed identity or completeness validation.
    #[error("invalid NNS neuron cache at {}: {reason}", path.display())]
    InvalidCache {
        /// Cache path being validated.
        path: PathBuf,
        /// Cache invariant that failed.
        reason: String,
    },

    /// Shared cache IO or lock handling failed.
    #[error(transparent)]
    Cache(#[from] HostCacheError),

    /// The synchronous host runtime could not execute the live query.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

#[cfg(feature = "host")]
fn enforce_mainnet_network(network: &str) -> Result<(), NnsNeuronHostError> {
    crate::network::enforce_mainnet_network_with(network, |network| {
        NnsNeuronHostError::UnsupportedNetwork { network }
    })
}
