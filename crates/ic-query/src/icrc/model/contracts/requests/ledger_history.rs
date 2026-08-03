//! Module: icrc::model::contracts::requests::ledger_history
//!
//! Responsibility: ICRC ledger-wide transaction and archive request contracts.
//! Does not own: account-index history, live transport, archive following, or reports.
//! Boundary: captures bounded ledger history and archive discovery intent.

///
/// IcrcTransactionsRequest
///
/// Request accepted by the generic ICRC transaction history report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcTransactionsRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
    pub start: u64,
    pub limit: u32,
    pub follow_archives: bool,
}

impl IcrcTransactionsRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        start: u64,
        limit: u32,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
            start,
            limit,
            follow_archives: false,
        }
    }

    #[must_use]
    pub const fn with_follow_archives(mut self, follow_archives: bool) -> Self {
        self.follow_archives = follow_archives;
        self
    }
}

///
/// IcrcArchivesRequest
///
/// Request accepted by the generic ICRC archives report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcArchivesRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
    pub from_canister_id: Option<String>,
}

impl IcrcArchivesRequest {
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
            from_canister_id: None,
        }
    }

    #[must_use]
    pub fn with_from_canister_id(mut self, from_canister_id: impl Into<String>) -> Self {
        self.from_canister_id = Some(from_canister_id.into());
        self
    }
}
