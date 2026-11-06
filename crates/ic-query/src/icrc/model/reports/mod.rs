//! Module: icrc::model::reports
//!
//! Responsibility: expose public serializable ICRC report and row contracts.
//! Does not own: requests, source data, errors, live transport, caching, or rendering.
//! Boundary: preserves one explicit report facade across cohesive capability families.

mod account;
mod account_history;
mod ledger;

pub use account::{IcrcAllowanceData, IcrcAllowanceReport, IcrcBalanceData, IcrcBalanceReport};
pub use account_history::{
    IcrcAccountRow, IcrcAccountTransactionCacheStatusReport, IcrcAccountTransactionCacheSummary,
    IcrcAccountTransactionContext, IcrcAccountTransactionListReport,
    IcrcAccountTransactionPageData, IcrcAccountTransactionPageReport,
    IcrcAccountTransactionRefreshAttemptStatus, IcrcAccountTransactionRefreshReport,
    IcrcAccountTransactionRow, IcrcAccountTransactionSnapshot,
    IcrcAccountTransactionValidCacheSummary,
};
pub use ledger::{
    IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow, IcrcArchivedRangeRow,
    IcrcArchivesReport, IcrcBlockTypeRow, IcrcBlockTypesReport, IcrcCapabilitiesReport,
    IcrcCapabilityRow, IcrcCapabilityStatus, IcrcFollowedArchiveBlockRow, IcrcIndexReport,
    IcrcTipCertificateEvidence, IcrcTipCertificateReport, IcrcTokenReport, IcrcTransactionBlockRow,
    IcrcTransactionsData, IcrcTransactionsReport,
};

///
/// IcrcReportProvenance
///
/// Ledger identity and acquisition provenance shared by live ICRC reports.
///

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct IcrcReportProvenance {
    /// Schema identifier of the containing report.
    pub schema_version: u32,
    /// Ledger canister queried by the caller.
    pub ledger_canister_id: String,
    /// UTC acquisition timestamp.
    pub fetched_at: String,
    /// Explicit IC API endpoint.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
}

#[cfg(feature = "icrc-host")]
impl IcrcReportProvenance {
    pub(in crate::icrc) fn from_request(
        request: &crate::icrc::IcrcLedgerRequest,
        schema_version: u32,
    ) -> Self {
        Self {
            schema_version,
            ledger_canister_id: request.ledger_canister_id.clone(),
            fetched_at: ic_host_text::timestamp::format_utc_timestamp_secs(request.now_unix_secs),
            source_endpoint: request.source_endpoint.clone(),
            fetched_by: crate::icrc::live::ICRC_FETCHED_BY.to_string(),
        }
    }
}
