//! Module: icrc
//!
//! Responsibility: top-level generic ICRC ledger query commands.
//! Does not own: SNS lookup, NNS registry cache behavior, or release flow.
//! Boundary: exposes live read-only token metadata, account balance, allowance,
//! index discovery, transaction history, block type, and archive reports.

#[cfg(feature = "cli")]
mod commands;
#[cfg(feature = "host")]
pub(crate) mod ledger;
#[cfg(feature = "host")]
mod live;
mod model;
mod text;

#[cfg(feature = "host")]
pub use live::{
    build_icrc_allowance_report, build_icrc_archives_report, build_icrc_balance_report,
    build_icrc_block_types_report, build_icrc_capabilities_report, build_icrc_index_report,
    build_icrc_tip_certificate_report, build_icrc_token_report, build_icrc_transactions_report,
};
pub use model::{
    IcrcAllowanceReport, IcrcAllowanceRequest, IcrcArchiveFollowErrorRow, IcrcArchiveRow,
    IcrcArchivedBlocksRow, IcrcArchivedRangeRow, IcrcArchivesReport, IcrcArchivesRequest,
    IcrcBalanceReport, IcrcBalanceRequest, IcrcBlockTypeRow, IcrcBlockTypesReport,
    IcrcBlockTypesRequest, IcrcCapabilitiesReport, IcrcCapabilitiesRequest, IcrcCapabilityRow,
    IcrcError, IcrcFollowedArchiveBlockRow, IcrcIndexReport, IcrcIndexRequest,
    IcrcTipCertificateReport, IcrcTipCertificateRequest, IcrcTokenMetadataRow, IcrcTokenReport,
    IcrcTokenRequest, IcrcTokenStandardRow, IcrcTransactionBlockRow, IcrcTransactionsReport,
    IcrcTransactionsRequest,
};
pub use text::{
    icrc_allowance_report_text, icrc_archives_report_text, icrc_balance_report_text,
    icrc_block_types_report_text, icrc_capabilities_report_text, icrc_index_report_text,
    icrc_tip_certificate_report_text, icrc_token_report_text, icrc_transactions_report_text,
};

#[cfg(feature = "cli")]
pub use commands::run;

#[cfg(all(test, feature = "cli", feature = "host"))]
mod tests;
