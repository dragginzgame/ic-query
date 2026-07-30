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
    runtime::block_on_current_thread,
};

///
/// IcrcTokenSource
///
/// Source capability for fetching generic ICRC token metadata.
///

pub trait IcrcTokenSource {
    fn fetch_token(&self, request: &IcrcLedgerRequest) -> Result<IcrcTokenData, IcrcError>;
}

///
/// IcrcBalanceSource
///
/// Source capability for fetching one ICRC account balance.
///

pub trait IcrcBalanceSource {
    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError>;
}

///
/// IcrcAllowanceSource
///
/// Source capability for fetching one ICRC account allowance.
///

pub trait IcrcAllowanceSource {
    fn fetch_allowance(
        &self,
        request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError>;
}

///
/// IcrcIndexSource
///
/// Source capability for discovering a ledger's ICRC index.
///

pub trait IcrcIndexSource {
    fn fetch_index(&self, request: &IcrcLedgerRequest) -> Result<IcrcIndexData, IcrcError>;
}

///
/// IcrcTransactionsSource
///
/// Source capability for fetching ICRC-3 ledger transaction blocks.
///

pub trait IcrcTransactionsSource {
    fn fetch_transactions(
        &self,
        request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError>;
}

///
/// IcrcBlockTypesSource
///
/// Source capability for fetching supported ICRC-3 block types.
///

pub trait IcrcBlockTypesSource {
    fn fetch_block_types(
        &self,
        request: &IcrcLedgerRequest,
    ) -> Result<IcrcBlockTypesData, IcrcError>;
}

///
/// IcrcArchivesSource
///
/// Source capability for discovering ICRC-3 archive ranges.
///

pub trait IcrcArchivesSource {
    fn fetch_archives(&self, request: &IcrcArchivesRequest) -> Result<IcrcArchivesData, IcrcError>;
}

///
/// IcrcTipCertificateSource
///
/// Source capability for fetching and verifying ICRC-3 tip evidence.
///

pub trait IcrcTipCertificateSource {
    fn fetch_tip_certificate(
        &self,
        request: &IcrcLedgerRequest,
    ) -> Result<IcrcTipCertificateData, IcrcError>;
}

///
/// IcrcCapabilitiesSource
///
/// Source capability for probing supported ledger and index operations.
///

pub trait IcrcCapabilitiesSource {
    fn fetch_capabilities(
        &self,
        request: &IcrcLedgerRequest,
    ) -> Result<IcrcCapabilitiesData, IcrcError>;
}

///
/// IcrcAccountTransactionPageSource
///
/// Source capability for resolving an ICRC index and fetching account history.
///

pub trait IcrcAccountTransactionPageSource {
    /// Fetches one backward page of transactions for the requested account.
    fn fetch_account_transaction_page(
        &self,
        request: &IcrcAccountTransactionPageRequest,
    ) -> Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError>;
}

///
/// IcrcAccountTransactionCollectionSource
///
/// Source capability for exhausting one verified ICRC account index.
///

pub trait IcrcAccountTransactionCollectionSource {
    /// Fetches complete account history without publishing a cache.
    fn fetch_complete_account_transactions(
        &self,
        request: &IcrcAccountTransactionRefreshRequest,
        progress: &mut (dyn QueryProgress + Send),
    ) -> Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError>;
}

///
/// LiveIcrcSource
///
/// Source implementation backed by live ICRC ledger canister queries.
/// Returned ICRC-3 tip evidence is authenticated and checked against the
/// ledger's certified-data value before it is exposed.
///

pub struct LiveIcrcSource;

impl IcrcTokenSource for LiveIcrcSource {
    fn fetch_token(&self, request: &IcrcLedgerRequest) -> Result<IcrcTokenData, IcrcError> {
        block_on_current_thread(fetch_token_async(request))?
    }
}

impl IcrcBalanceSource for LiveIcrcSource {
    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
        block_on_current_thread(fetch_balance_async(request))?
    }
}

impl IcrcAllowanceSource for LiveIcrcSource {
    fn fetch_allowance(
        &self,
        request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError> {
        block_on_current_thread(fetch_allowance_async(request))?
    }
}

impl IcrcIndexSource for LiveIcrcSource {
    fn fetch_index(&self, request: &IcrcLedgerRequest) -> Result<IcrcIndexData, IcrcError> {
        block_on_current_thread(fetch_index_async(request))?
    }
}

impl IcrcTransactionsSource for LiveIcrcSource {
    fn fetch_transactions(
        &self,
        request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError> {
        block_on_current_thread(fetch_transactions_async(request))?
    }
}

impl IcrcBlockTypesSource for LiveIcrcSource {
    fn fetch_block_types(
        &self,
        request: &IcrcLedgerRequest,
    ) -> Result<IcrcBlockTypesData, IcrcError> {
        block_on_current_thread(fetch_block_types_async(request))?
    }
}

impl IcrcArchivesSource for LiveIcrcSource {
    fn fetch_archives(&self, request: &IcrcArchivesRequest) -> Result<IcrcArchivesData, IcrcError> {
        block_on_current_thread(fetch_archives_async(request))?
    }
}

impl IcrcTipCertificateSource for LiveIcrcSource {
    fn fetch_tip_certificate(
        &self,
        request: &IcrcLedgerRequest,
    ) -> Result<IcrcTipCertificateData, IcrcError> {
        block_on_current_thread(fetch_tip_certificate_async(request))?
    }
}

impl IcrcCapabilitiesSource for LiveIcrcSource {
    fn fetch_capabilities(
        &self,
        request: &IcrcLedgerRequest,
    ) -> Result<IcrcCapabilitiesData, IcrcError> {
        block_on_current_thread(fetch_capabilities_async(request))?
    }
}

impl IcrcAccountTransactionPageSource for LiveIcrcSource {
    fn fetch_account_transaction_page(
        &self,
        request: &IcrcAccountTransactionPageRequest,
    ) -> Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError> {
        block_on_current_thread(fetch_account_transaction_page_async(request))
            .map_err(IcrcError::from)?
    }
}

impl IcrcAccountTransactionCollectionSource for LiveIcrcSource {
    fn fetch_complete_account_transactions(
        &self,
        request: &IcrcAccountTransactionRefreshRequest,
        progress: &mut (dyn QueryProgress + Send),
    ) -> Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError> {
        block_on_current_thread(fetch_complete_account_transactions_async(request, progress))
            .map_err(IcrcError::from)?
    }
}
