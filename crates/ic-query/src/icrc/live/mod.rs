//! Module: icrc::live
//!
//! Responsibility: expose ICRC report builders and live source adapters.
//! Does not own: command parsing, text rendering, or cache behavior.
//! Boundary: keeps report assembly, source adaptation, and host calls in separate owners.

pub(in crate::icrc) mod account_transactions;
pub(in crate::icrc) mod build;
mod fetch;
pub(in crate::icrc) mod source;
mod tip_certificate;

use crate::icrc::model::IcrcError;

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

crate::candid_call::impl_candid_call_error!(IcrcError);
