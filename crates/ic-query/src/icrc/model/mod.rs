//! Module: icrc::model
//!
//! Responsibility: expose typed ICRC requests, reports, source data, and errors.
//! Does not own: Clap parsing, live transport, report construction, or rendering.
//! Boundary: keeps public contracts, host-source data, errors, and validation separate.

mod contracts;
#[cfg(feature = "host")]
mod data;
mod error;
mod subaccount;

pub use contracts::{
    IcrcAccountRow, IcrcAccountTransactionCacheRequest, IcrcAccountTransactionCacheStatusReport,
    IcrcAccountTransactionCacheSummary, IcrcAccountTransactionListReport,
    IcrcAccountTransactionListRequest, IcrcAccountTransactionPageReport,
    IcrcAccountTransactionPageRequest, IcrcAccountTransactionRefreshAttemptStatus,
    IcrcAccountTransactionRefreshReport, IcrcAccountTransactionRefreshRequest,
    IcrcAccountTransactionRow, IcrcAccountTransactionSnapshot, IcrcAccountTransactionSort,
    IcrcAllowanceReport, IcrcAllowanceRequest, IcrcArchiveFollowErrorRow, IcrcArchiveRow,
    IcrcArchivedBlocksRow, IcrcArchivedRangeRow, IcrcArchivesReport, IcrcArchivesRequest,
    IcrcBalanceReport, IcrcBalanceRequest, IcrcBlockTypeRow, IcrcBlockTypesReport,
    IcrcCapabilitiesReport, IcrcCapabilityRow, IcrcFollowedArchiveBlockRow, IcrcIndexReport,
    IcrcLedgerRequest, IcrcTipCertificateReport, IcrcTokenMetadataRow, IcrcTokenReport,
    IcrcTokenStandardRow, IcrcTransactionBlockRow, IcrcTransactionsReport, IcrcTransactionsRequest,
};
#[cfg(feature = "host")]
pub use data::{
    CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionCollectionData,
    IcrcAccountTransactionPageData, IcrcAllowanceData, IcrcArchivesData, IcrcBalanceData,
    IcrcBlockTypesData, IcrcCapabilitiesData, IcrcIndexData, IcrcTipCertificateData, IcrcTokenData,
    IcrcTransactionsData,
};
pub use error::{IcrcAccountTransactionError, IcrcError};
pub use subaccount::normalize_subaccount_hex;
#[cfg(feature = "host")]
pub(in crate::icrc) use subaccount::{
    normalize_optional_subaccount_hex, subaccount_bytes_from_hex,
};
