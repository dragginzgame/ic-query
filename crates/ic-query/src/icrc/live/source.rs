//! Module: icrc::live::source
//!
//! Responsibility: define the ICRC source contract and live synchronous adapter.
//! Does not own: Candid host calls, report construction, or text rendering.
//! Boundary: isolates the synchronous public source API from async live fetching.

use super::fetch::{
    fetch_allowance_async, fetch_archives_async, fetch_balance_async, fetch_block_types_async,
    fetch_capabilities_async, fetch_index_async, fetch_tip_certificate_async, fetch_token_async,
    fetch_transactions_async,
};
use crate::{
    icrc::model::{
        IcrcAllowanceData, IcrcAllowanceRequest, IcrcArchivesData, IcrcArchivesRequest,
        IcrcBalanceData, IcrcBalanceRequest, IcrcBlockTypesData, IcrcBlockTypesRequest,
        IcrcCapabilitiesData, IcrcCapabilitiesRequest, IcrcError, IcrcIndexData, IcrcIndexRequest,
        IcrcTipCertificateData, IcrcTipCertificateRequest, IcrcTokenData, IcrcTokenRequest,
        IcrcTransactionsData, IcrcTransactionsRequest,
    },
    runtime::block_on_current_thread,
};

///
/// IcrcSource
///
/// Source contract for fetching generic ICRC ledger metadata, balances, allowances, indexes, and ICRC-3 rows.
///

pub trait IcrcSource {
    fn fetch_token(&self, request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError>;

    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError>;

    fn fetch_allowance(
        &self,
        request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError>;

    fn fetch_index(&self, request: &IcrcIndexRequest) -> Result<IcrcIndexData, IcrcError>;

    fn fetch_transactions(
        &self,
        request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError>;

    fn fetch_block_types(
        &self,
        request: &IcrcBlockTypesRequest,
    ) -> Result<IcrcBlockTypesData, IcrcError>;

    fn fetch_archives(&self, request: &IcrcArchivesRequest) -> Result<IcrcArchivesData, IcrcError>;

    fn fetch_tip_certificate(
        &self,
        request: &IcrcTipCertificateRequest,
    ) -> Result<IcrcTipCertificateData, IcrcError>;

    fn fetch_capabilities(
        &self,
        request: &IcrcCapabilitiesRequest,
    ) -> Result<IcrcCapabilitiesData, IcrcError>;
}

///
/// LiveIcrcSource
///
/// Source implementation backed by live ICRC ledger canister queries.
///

pub struct LiveIcrcSource;

impl IcrcSource for LiveIcrcSource {
    fn fetch_token(&self, request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError> {
        block_on_current_thread(fetch_token_async(request))?
    }

    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
        block_on_current_thread(fetch_balance_async(request))?
    }

    fn fetch_allowance(
        &self,
        request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError> {
        block_on_current_thread(fetch_allowance_async(request))?
    }

    fn fetch_index(&self, request: &IcrcIndexRequest) -> Result<IcrcIndexData, IcrcError> {
        block_on_current_thread(fetch_index_async(request))?
    }

    fn fetch_transactions(
        &self,
        request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError> {
        block_on_current_thread(fetch_transactions_async(request))?
    }

    fn fetch_block_types(
        &self,
        request: &IcrcBlockTypesRequest,
    ) -> Result<IcrcBlockTypesData, IcrcError> {
        block_on_current_thread(fetch_block_types_async(request))?
    }

    fn fetch_archives(&self, request: &IcrcArchivesRequest) -> Result<IcrcArchivesData, IcrcError> {
        block_on_current_thread(fetch_archives_async(request))?
    }

    fn fetch_tip_certificate(
        &self,
        request: &IcrcTipCertificateRequest,
    ) -> Result<IcrcTipCertificateData, IcrcError> {
        block_on_current_thread(fetch_tip_certificate_async(request))?
    }

    fn fetch_capabilities(
        &self,
        request: &IcrcCapabilitiesRequest,
    ) -> Result<IcrcCapabilitiesData, IcrcError> {
        block_on_current_thread(fetch_capabilities_async(request))?
    }
}
