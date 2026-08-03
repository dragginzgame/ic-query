//! Module: icrc::model::contracts::reports
//!
//! Responsibility: expose public serializable ICRC report and row contracts.
//! Does not own: requests, source data, errors, live transport, caching, or rendering.
//! Boundary: preserves one explicit report facade across cohesive capability families.

mod account;
mod account_history;
mod ledger;

pub use account::{IcrcAllowanceReport, IcrcBalanceReport};
pub use account_history::{
    IcrcAccountRow, IcrcAccountTransactionCacheStatusReport, IcrcAccountTransactionCacheSummary,
    IcrcAccountTransactionListReport, IcrcAccountTransactionPageReport,
    IcrcAccountTransactionRefreshAttemptStatus, IcrcAccountTransactionRefreshReport,
    IcrcAccountTransactionRow, IcrcAccountTransactionSnapshot,
};
pub use ledger::{
    IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow, IcrcArchivedRangeRow,
    IcrcArchivesReport, IcrcBlockTypeRow, IcrcBlockTypesReport, IcrcCapabilitiesReport,
    IcrcCapabilityRow, IcrcCapabilityStatus, IcrcFollowedArchiveBlockRow, IcrcIndexReport,
    IcrcMetadataValueKind, IcrcTipCertificateReport, IcrcTokenMetadataRow, IcrcTokenReport,
    IcrcTokenStandardRow, IcrcTransactionBlockRow, IcrcTransactionsReport,
};
