//! Module: icrc::model
//!
//! Responsibility: expose typed ICRC requests, reports, source data, and errors.
//! Does not own: Clap parsing, live transport, report construction, or rendering.
//! Boundary: keeps public contracts, host-source data, errors, and validation separate.

mod contracts;
#[cfg(feature = "icrc-host")]
mod data;
mod error;
mod subaccount;
mod token_metadata;

pub use contracts::{
    IcrcAccountRow, IcrcAccountTransactionCacheRequest, IcrcAccountTransactionCacheStatusReport,
    IcrcAccountTransactionCacheSummary, IcrcAccountTransactionListReport,
    IcrcAccountTransactionListRequest, IcrcAccountTransactionPageData,
    IcrcAccountTransactionPageReport, IcrcAccountTransactionPageRequest,
    IcrcAccountTransactionRefreshAttemptStatus, IcrcAccountTransactionRefreshReport,
    IcrcAccountTransactionRefreshRequest, IcrcAccountTransactionRow,
    IcrcAccountTransactionSnapshot, IcrcAccountTransactionSort, IcrcAllowanceData,
    IcrcAllowanceReport, IcrcAllowanceRequest, IcrcArchiveFollowErrorRow, IcrcArchiveRow,
    IcrcArchivedBlocksRow, IcrcArchivedRangeRow, IcrcArchivesReport, IcrcArchivesRequest,
    IcrcBalanceData, IcrcBalanceReport, IcrcBalanceRequest, IcrcBlockTypeRow, IcrcBlockTypesReport,
    IcrcCapabilitiesReport, IcrcCapabilityRow, IcrcCapabilityStatus, IcrcFollowedArchiveBlockRow,
    IcrcIndexReport, IcrcLedgerRequest, IcrcTipCertificateReport, IcrcTokenReport,
    IcrcTransactionBlockRow, IcrcTransactionsReport, IcrcTransactionsRequest,
};
#[cfg(feature = "icrc-host")]
pub use data::{
    CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionCollectionData,
    IcrcCapabilitiesData, IcrcTransactionsData,
};
pub use error::{IcrcAccountTransactionError, IcrcError};
pub use subaccount::normalize_subaccount_hex;
#[cfg(feature = "icrc-host")]
pub(in crate::icrc) use subaccount::{
    normalize_optional_subaccount_hex, subaccount_bytes_from_hex,
};

pub use token_metadata::{
    IcrcIndexData, IcrcMetadataValueKind, IcrcTokenData, IcrcTokenMetadataRow, IcrcTokenStandardRow,
};
