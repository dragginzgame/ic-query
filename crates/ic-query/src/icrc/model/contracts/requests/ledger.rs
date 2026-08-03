//! Module: icrc::model::contracts::requests::ledger
//!
//! Responsibility: shared ICRC ledger identity and provenance requests.
//! Does not own: account selection, history pagination, live transport, or reports.
//! Boundary: captures the common ledger target used by point-in-time report builders.

///
/// IcrcLedgerRequest
///
/// Shared ledger identity and provenance for metadata and capability report builders.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcLedgerRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
}

impl IcrcLedgerRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
        }
    }
}
