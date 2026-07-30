//! Module: icrc::live
//!
//! Responsibility: expose ICRC report builders and live source adapters.
//! Does not own: command parsing, text rendering, or cache behavior.
//! Boundary: keeps report assembly, source adaptation, and host calls in separate owners.

mod account_transactions;
mod build;
mod fetch;
mod source;
mod tip_certificate;

pub use build::{
    build_icrc_account_transactions_report, build_icrc_account_transactions_report_with_source,
    build_icrc_allowance_report, build_icrc_allowance_report_with_source,
    build_icrc_archives_report, build_icrc_archives_report_with_source, build_icrc_balance_report,
    build_icrc_balance_report_with_source, build_icrc_block_types_report,
    build_icrc_block_types_report_with_source, build_icrc_capabilities_report,
    build_icrc_capabilities_report_with_source, build_icrc_index_report,
    build_icrc_index_report_with_source, build_icrc_tip_certificate_report,
    build_icrc_tip_certificate_report_with_source, build_icrc_token_report,
    build_icrc_token_report_with_source, build_icrc_transactions_report,
    build_icrc_transactions_report_with_source,
};
pub use source::{IcrcAccountTransactionsSource, IcrcSource, LiveIcrcSource};

use crate::icrc::{ledger::IcrcLedgerError, model::IcrcError};

pub(in crate::icrc) const ICRC_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_BALANCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_ACCOUNT_TRANSACTIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_INDEX_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_TRANSACTIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_BLOCK_TYPES_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_ARCHIVES_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_TIP_CERTIFICATE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_CAPABILITIES_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_FETCHED_BY: &str = "ic-query";

impl IcrcLedgerError for IcrcError {
    fn agent_build(endpoint: &str, reason: String) -> Self {
        Self::AgentBuild {
            endpoint: endpoint.to_string(),
            reason,
        }
    }

    fn invalid_principal(field: &'static str, reason: String) -> Self {
        Self::InvalidPrincipal { field, reason }
    }

    fn candid_encode(message: &'static str, reason: String) -> Self {
        Self::CandidEncode { message, reason }
    }

    fn agent_call(method: &'static str, reason: String) -> Self {
        Self::AgentCall { method, reason }
    }

    fn candid_decode(message: &'static str, reason: String) -> Self {
        Self::CandidDecode { message, reason }
    }
}
