//! Module: icrc::live::source
//!
//! Responsibility: define the ICRC source contract and live synchronous adapter.
//! Does not own: Candid host calls, report construction, or text rendering.
//! Boundary: isolates the synchronous public source API from async live fetching.

use super::{
    account_transactions::{
        fetch_account_transaction_page_async, fetch_complete_account_transactions_async,
    },
    fetch::{
        fetch_allowance_async, fetch_archives_async, fetch_balance_async, fetch_block_types_async,
        fetch_capabilities_async, fetch_index_async, fetch_tip_certificate_async,
        fetch_token_async, fetch_transactions_async,
    },
};
use crate::{
    QueryProgress,
    icrc::model::{
        IcrcAccountTransactionCollectionData, IcrcAccountTransactionError,
        IcrcAccountTransactionPageData, IcrcAccountTransactionPageRequest,
        IcrcAccountTransactionRefreshRequest, IcrcAllowanceData, IcrcAllowanceRequest,
        IcrcArchiveRow, IcrcArchivesRequest, IcrcBalanceData, IcrcBalanceRequest, IcrcBlockTypeRow,
        IcrcCapabilitiesData, IcrcError, IcrcIndexData, IcrcLedgerRequest, IcrcTokenData,
        IcrcTransactionsData, IcrcTransactionsRequest,
    },
};
use icrc_ledger_types::icrc3::blocks::ICRC3DataCertificate;

///
/// IcrcTokenSource
///
/// Source capability for fetching generic ICRC token metadata.
///

pub trait IcrcTokenSource: Sync {
    /// Fetch raw token metadata and its standard ledger attributes.
    fn fetch_token<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcTokenData, IcrcError>>;
}

///
/// IcrcBalanceSource
///
/// Source capability for fetching one ICRC account balance.
///

pub trait IcrcBalanceSource: Sync {
    /// Fetch the requested account balance in raw base units.
    fn fetch_balance<'a>(
        &'a self,
        request: &'a IcrcBalanceRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcBalanceData, IcrcError>>;
}

///
/// IcrcAllowanceSource
///
/// Source capability for fetching one ICRC account allowance.
///

pub trait IcrcAllowanceSource: Sync {
    /// Fetch the owner-to-spender allowance and optional expiry.
    fn fetch_allowance<'a>(
        &'a self,
        request: &'a IcrcAllowanceRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcAllowanceData, IcrcError>>;
}

///
/// IcrcIndexSource
///
/// Source capability for discovering a ledger's ICRC index.
///

pub trait IcrcIndexSource: Sync {
    /// Discover the ledger index through ICRC-106.
    fn fetch_index<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcIndexData, IcrcError>>;
}

///
/// IcrcTransactionsSource
///
/// Source capability for fetching ICRC-3 ledger transaction blocks.
///

pub trait IcrcTransactionsSource: Sync {
    /// Fetch a bounded ledger block range and follow callbacks only when requested.
    /// Ledger blocks and archive ranges must be disjoint and inside the requested page.
    /// Followed blocks must have unique ids and match a returned callback's ranges;
    /// follow errors must identify a returned callback. At most 100 callbacks are accepted.
    fn fetch_transactions<'a>(
        &'a self,
        request: &'a IcrcTransactionsRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcTransactionsData, IcrcError>>;
}

///
/// IcrcBlockTypesSource
///
/// Source capability for fetching supported ICRC-3 block types.
///

pub trait IcrcBlockTypesSource: Sync {
    /// Fetch the block type identifiers and specification URLs advertised by the ledger.
    fn fetch_block_types<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<Vec<IcrcBlockTypeRow>, IcrcError>>;
}

///
/// IcrcArchivesSource
///
/// Source capability for discovering ICRC-3 archive ranges.
///

pub trait IcrcArchivesSource: Sync {
    /// Fetch archive coverage beginning after the optional archive cursor.
    fn fetch_archives<'a>(
        &'a self,
        request: &'a IcrcArchivesRequest,
    ) -> crate::report::SourceFuture<'a, Result<Vec<IcrcArchiveRow>, IcrcError>>;
}

///
/// IcrcTipCertificateSource
///
/// Source capability for fetching and verifying ICRC-3 tip evidence.
///

pub trait IcrcTipCertificateSource: Sync {
    /// Fetch and authenticate tip certificate evidence when provided by the ledger.
    fn fetch_tip_certificate<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<Option<ICRC3DataCertificate>, IcrcError>>;
}

///
/// IcrcCapabilitiesSource
///
/// Source capability for probing supported ledger and index operations.
///

pub trait IcrcCapabilitiesSource: Sync {
    /// Probe ledger and discovered-index methods, retaining per-capability failures.
    fn fetch_capabilities<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcCapabilitiesData, IcrcError>>;
}

///
/// IcrcAccountTransactionPageSource
///
/// Source capability for resolving an ICRC index and fetching account history.
///

pub trait IcrcAccountTransactionPageSource: Sync {
    /// Fetches one backward page of transactions for the requested account.
    fn fetch_account_transaction_page<'a>(
        &'a self,
        request: &'a IcrcAccountTransactionPageRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError>,
    >;
}

///
/// IcrcAccountTransactionCollectionSource
///
/// Source capability for exhausting one verified ICRC account index.
///

pub trait IcrcAccountTransactionCollectionSource: Sync {
    /// Fetches complete account history without publishing a cache.
    fn fetch_complete_account_transactions<'a>(
        &'a self,
        request: &'a IcrcAccountTransactionRefreshRequest,
        progress: &'a mut (dyn QueryProgress + Send),
    ) -> crate::report::SourceFuture<
        'a,
        Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError>,
    >;
}

///
/// LiveIcrcSource
///
/// Source implementation backed by live ICRC ledger canister queries.
/// Returned ICRC-3 tip evidence is authenticated and checked against the
/// ledger's certified-data value before it is exposed.
///

#[derive(Clone, Debug, Default)]
pub struct LiveIcrcSource {
    pub(crate) agent: crate::agent::EndpointAgent,
}

impl IcrcTokenSource for LiveIcrcSource {
    fn fetch_token<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcTokenData, IcrcError>> {
        Box::pin(fetch_token_async(self, request))
    }
}

impl IcrcBalanceSource for LiveIcrcSource {
    fn fetch_balance<'a>(
        &'a self,
        request: &'a IcrcBalanceRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcBalanceData, IcrcError>> {
        Box::pin(fetch_balance_async(self, request))
    }
}

impl IcrcAllowanceSource for LiveIcrcSource {
    fn fetch_allowance<'a>(
        &'a self,
        request: &'a IcrcAllowanceRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcAllowanceData, IcrcError>> {
        Box::pin(fetch_allowance_async(self, request))
    }
}

impl IcrcIndexSource for LiveIcrcSource {
    fn fetch_index<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcIndexData, IcrcError>> {
        Box::pin(fetch_index_async(self, request))
    }
}

impl IcrcTransactionsSource for LiveIcrcSource {
    fn fetch_transactions<'a>(
        &'a self,
        request: &'a IcrcTransactionsRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcTransactionsData, IcrcError>> {
        Box::pin(fetch_transactions_async(self, request))
    }
}

impl IcrcBlockTypesSource for LiveIcrcSource {
    fn fetch_block_types<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<Vec<IcrcBlockTypeRow>, IcrcError>> {
        Box::pin(fetch_block_types_async(self, request))
    }
}

impl IcrcArchivesSource for LiveIcrcSource {
    fn fetch_archives<'a>(
        &'a self,
        request: &'a IcrcArchivesRequest,
    ) -> crate::report::SourceFuture<'a, Result<Vec<IcrcArchiveRow>, IcrcError>> {
        Box::pin(fetch_archives_async(self, request))
    }
}

impl IcrcTipCertificateSource for LiveIcrcSource {
    fn fetch_tip_certificate<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<Option<ICRC3DataCertificate>, IcrcError>> {
        Box::pin(fetch_tip_certificate_async(self, request))
    }
}

impl IcrcCapabilitiesSource for LiveIcrcSource {
    fn fetch_capabilities<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcCapabilitiesData, IcrcError>> {
        Box::pin(fetch_capabilities_async(self, request))
    }
}

impl IcrcAccountTransactionPageSource for LiveIcrcSource {
    fn fetch_account_transaction_page<'a>(
        &'a self,
        request: &'a IcrcAccountTransactionPageRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError>,
    > {
        Box::pin(fetch_account_transaction_page_async(self, request))
    }
}

impl IcrcAccountTransactionCollectionSource for LiveIcrcSource {
    fn fetch_complete_account_transactions<'a>(
        &'a self,
        request: &'a IcrcAccountTransactionRefreshRequest,
        progress: &'a mut (dyn QueryProgress + Send),
    ) -> crate::report::SourceFuture<
        'a,
        Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError>,
    > {
        Box::pin(fetch_complete_account_transactions_async(
            self, request, progress,
        ))
    }
}
