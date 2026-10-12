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
        IcrcArchivesData, IcrcArchivesRequest, IcrcBalanceData, IcrcBalanceRequest,
        IcrcBlockTypesData, IcrcCapabilitiesData, IcrcError, IcrcIndexData, IcrcLedgerRequest,
        IcrcTipCertificateData, IcrcTokenData, IcrcTransactionsData, IcrcTransactionsRequest,
    },
};

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
    ) -> crate::report::SourceFuture<'a, Result<IcrcBlockTypesData, IcrcError>>;
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
    ) -> crate::report::SourceFuture<'a, Result<IcrcArchivesData, IcrcError>>;
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
    ) -> crate::report::SourceFuture<'a, Result<IcrcTipCertificateData, IcrcError>>;
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
        Box::pin(async move { fetch_token_async(self, request).await })
    }
}

impl IcrcBalanceSource for LiveIcrcSource {
    fn fetch_balance<'a>(
        &'a self,
        request: &'a IcrcBalanceRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcBalanceData, IcrcError>> {
        Box::pin(async move { fetch_balance_async(self, request).await })
    }
}

impl IcrcAllowanceSource for LiveIcrcSource {
    fn fetch_allowance<'a>(
        &'a self,
        request: &'a IcrcAllowanceRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcAllowanceData, IcrcError>> {
        Box::pin(async move { fetch_allowance_async(self, request).await })
    }
}

impl IcrcIndexSource for LiveIcrcSource {
    fn fetch_index<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcIndexData, IcrcError>> {
        Box::pin(async move { fetch_index_async(self, request).await })
    }
}

impl IcrcTransactionsSource for LiveIcrcSource {
    fn fetch_transactions<'a>(
        &'a self,
        request: &'a IcrcTransactionsRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcTransactionsData, IcrcError>> {
        Box::pin(async move { fetch_transactions_async(self, request).await })
    }
}

impl IcrcBlockTypesSource for LiveIcrcSource {
    fn fetch_block_types<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcBlockTypesData, IcrcError>> {
        Box::pin(async move { fetch_block_types_async(self, request).await })
    }
}

impl IcrcArchivesSource for LiveIcrcSource {
    fn fetch_archives<'a>(
        &'a self,
        request: &'a IcrcArchivesRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcArchivesData, IcrcError>> {
        Box::pin(async move { fetch_archives_async(self, request).await })
    }
}

impl IcrcTipCertificateSource for LiveIcrcSource {
    fn fetch_tip_certificate<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcTipCertificateData, IcrcError>> {
        Box::pin(async move { fetch_tip_certificate_async(self, request).await })
    }
}

impl IcrcCapabilitiesSource for LiveIcrcSource {
    fn fetch_capabilities<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcrcCapabilitiesData, IcrcError>> {
        Box::pin(async move { fetch_capabilities_async(self, request).await })
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
        Box::pin(async move { fetch_account_transaction_page_async(self, request).await })
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
        Box::pin(
            async move { fetch_complete_account_transactions_async(self, request, progress).await },
        )
    }
}
