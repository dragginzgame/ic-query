//! Module: icrc::model::contracts
//!
//! Responsibility: expose public ICRC request, report, and row contracts.
//! Does not own: errors, source data, subaccount validation, live transport, or rendering.
//! Boundary: preserves the existing model API while request and response ownership remain separate.

mod reports;
mod requests;

pub use reports::{
    IcrcAccountRow, IcrcAccountTransactionCacheStatusReport, IcrcAccountTransactionCacheSummary,
    IcrcAccountTransactionListReport, IcrcAccountTransactionPageReport,
    IcrcAccountTransactionRefreshAttemptStatus, IcrcAccountTransactionRefreshReport,
    IcrcAccountTransactionRow, IcrcAccountTransactionSnapshot, IcrcAllowanceReport,
    IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow, IcrcArchivedRangeRow,
    IcrcArchivesReport, IcrcBalanceReport, IcrcBlockTypeRow, IcrcBlockTypesReport,
    IcrcCapabilitiesReport, IcrcCapabilityRow, IcrcFollowedArchiveBlockRow, IcrcIndexReport,
    IcrcTipCertificateReport, IcrcTokenMetadataRow, IcrcTokenReport, IcrcTokenStandardRow,
    IcrcTransactionBlockRow, IcrcTransactionsReport,
};
pub use requests::{
    IcrcAccountTransactionCacheRequest, IcrcAccountTransactionListRequest,
    IcrcAccountTransactionPageRequest, IcrcAccountTransactionRefreshRequest,
    IcrcAccountTransactionSort, IcrcAllowanceRequest, IcrcArchivesRequest, IcrcBalanceRequest,
    IcrcLedgerRequest, IcrcTransactionsRequest,
};
