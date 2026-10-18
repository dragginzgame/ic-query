//! Module: icrc::live
//!
//! Responsibility: expose ICRC report builders and live source adapters.
//! Does not own: command parsing, text rendering, or cache behavior.
//! Boundary: keeps report assembly, source adaptation, and host calls in separate owners.

pub(in crate::icrc) mod account_transactions;
mod build;
mod fetch;
mod source;
mod tip_certificate;

pub use build::{
    build_icrc_account_transaction_page_report,
    build_icrc_account_transaction_page_report_with_source, build_icrc_allowance_report,
    build_icrc_allowance_report_with_source, build_icrc_archives_report,
    build_icrc_archives_report_with_source, build_icrc_balance_report,
    build_icrc_balance_report_with_source, build_icrc_block_types_report,
    build_icrc_block_types_report_with_source, build_icrc_capabilities_report,
    build_icrc_capabilities_report_with_source, build_icrc_index_report,
    build_icrc_index_report_with_source, build_icrc_tip_certificate_report,
    build_icrc_tip_certificate_report_with_source, build_icrc_token_report,
    build_icrc_token_report_with_source, build_icrc_transactions_report,
    build_icrc_transactions_report_with_source,
};
pub use source::{
    IcrcAccountTransactionCollectionSource, IcrcAccountTransactionPageSource, IcrcAllowanceSource,
    IcrcArchivesSource, IcrcBalanceSource, IcrcBlockTypesSource, IcrcCapabilitiesSource,
    IcrcIndexSource, IcrcTipCertificateSource, IcrcTokenSource, IcrcTransactionsSource,
    LiveIcrcSource,
};

use crate::{candid_call::CandidCallError, icrc::model::IcrcError};
use ic_platform::agent::AgentError;

pub(in crate::icrc) const ICRC_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_BALANCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_ACCOUNT_TRANSACTION_PAGE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_INDEX_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_TRANSACTIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_BLOCK_TYPES_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_ARCHIVES_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_TIP_CERTIFICATE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_CAPABILITIES_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_FETCHED_BY: &str = "ic-query";

/// Largest account-history page accepted by the built-in index adapter.
pub const ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE: u32 = 100;

impl CandidCallError for IcrcError {
    fn candid_encode(message: &'static str, reason: String) -> Self {
        Self::CandidEncode { message, reason }
    }

    fn agent_call(method: &'static str, error: AgentError) -> Self {
        Self::AgentCall {
            method,
            reason: error.to_string(),
        }
    }

    fn candid_decode(message: &'static str, reason: String) -> Self {
        Self::CandidDecode { message, reason }
    }
}
