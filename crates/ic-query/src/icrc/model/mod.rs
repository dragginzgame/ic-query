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
    IcrcAccountRow, IcrcAccountTransactionRow, IcrcAccountTransactionsReport,
    IcrcAccountTransactionsRequest, IcrcAllowanceReport, IcrcAllowanceRequest,
    IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow, IcrcArchivedRangeRow,
    IcrcArchivesReport, IcrcArchivesRequest, IcrcBalanceReport, IcrcBalanceRequest,
    IcrcBlockTypeRow, IcrcBlockTypesReport, IcrcBlockTypesRequest, IcrcCapabilitiesReport,
    IcrcCapabilitiesRequest, IcrcCapabilityRow, IcrcFollowedArchiveBlockRow, IcrcIndexReport,
    IcrcIndexRequest, IcrcTipCertificateReport, IcrcTipCertificateRequest, IcrcTokenMetadataRow,
    IcrcTokenReport, IcrcTokenRequest, IcrcTokenStandardRow, IcrcTransactionBlockRow,
    IcrcTransactionsReport, IcrcTransactionsRequest,
};
#[cfg(feature = "host")]
pub use data::{
    IcrcAccountTransactionsData, IcrcAllowanceData, IcrcArchivesData, IcrcBalanceData,
    IcrcBlockTypesData, IcrcCapabilitiesData, IcrcIndexData, IcrcTipCertificateData, IcrcTokenData,
    IcrcTransactionsData,
};
pub use error::{IcrcAccountTransactionsError, IcrcError};
pub use subaccount::normalize_subaccount_hex;
#[cfg(feature = "host")]
pub(in crate::icrc) use subaccount::subaccount_bytes_from_hex;
