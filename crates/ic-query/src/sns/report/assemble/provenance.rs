//! Module: sns::report::assemble::provenance
//!
//! Responsibility: build common live/cache provenance fields for SNS reports.
//! Does not own: cache loading, report DTOs, or text rendering.
//! Boundary: keeps report-source metadata construction consistent.

use crate::{
    report::ReportDataSource,
    sns::report::{MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, SnsSourceRequest},
};
use std::path::Path;

///
/// SnsReportProvenance
///
/// Shared source metadata attached to SNS reports.
///

pub(in crate::sns::report) struct SnsReportProvenance {
    pub(in crate::sns::report) data_source: ReportDataSource,
    pub(in crate::sns::report) cache_path: Option<String>,
    pub(in crate::sns::report) cache_complete: Option<bool>,
}

impl SnsReportProvenance {
    /// Build provenance for live SNS source reports.
    pub(in crate::sns::report) const fn live() -> Self {
        Self {
            data_source: ReportDataSource::Live,
            cache_path: None,
            cache_complete: None,
        }
    }

    /// Build provenance for complete-cache SNS source reports.
    pub(in crate::sns::report) fn cache(cache_path: &Path, cache_complete: bool) -> Self {
        Self {
            data_source: ReportDataSource::Cache,
            cache_path: Some(cache_path.display().to_string()),
            cache_complete: Some(cache_complete),
        }
    }
}

///
/// SnsReportContext
///
/// Validated identity and acquisition metadata shared by SNS list and detail reports.
///

pub(in crate::sns::report) struct SnsReportContext {
    pub(in crate::sns::report) network: String,
    pub(in crate::sns::report) sns_wasm_canister_id: String,
    pub(in crate::sns::report) fetched_at: String,
    pub(in crate::sns::report) source_endpoint: String,
    pub(in crate::sns::report) fetched_by: String,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) name: String,
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
}

impl SnsReportContext {
    /// Project the validated live source identity and collection provenance.
    pub(in crate::sns::report) fn from_live(
        fetch_request: SnsSourceRequest,
        sns: MainnetSns,
    ) -> Self {
        Self {
            network: fetch_request.network,
            sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
            fetched_at: fetch_request.fetched_at,
            source_endpoint: fetch_request.endpoint,
            fetched_by: fetch_request.fetched_by,
            id: sns.id,
            name: sns.name,
            root_canister_id: sns.root_canister_id,
            governance_canister_id: sns.governance_canister_id,
        }
    }
}
