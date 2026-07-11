//! Module: icrc::commands::test_support
//!
//! Responsibility: expose typed ICRC command parsers and usage to unit tests.
//! Does not own: production parsing, dispatch, or behavior assertions.
//! Boundary: keeps test-only access out of the production command surface.

use super::{
    IcrcAllowanceOptions, IcrcArchivesOptions, IcrcBalanceOptions, IcrcBlockTypesOptions,
    IcrcCapabilitiesOptions, IcrcIndexOptions, IcrcTipCertificateOptions, IcrcTokenOptions,
    IcrcTransactionsOptions, icrc_allowance_usage, icrc_archives_usage, icrc_balance_usage,
    icrc_block_types_usage, icrc_capabilities_usage, icrc_index_usage, icrc_tip_certificate_usage,
    icrc_token_usage, icrc_transactions_usage, usage,
};

pub(in crate::icrc) fn parse_token_options(args: &[&str]) -> IcrcTokenOptions {
    try_parse_token_options(args).expect("parse ICRC token options")
}

pub(in crate::icrc) fn try_parse_token_options(
    args: &[&str],
) -> Result<IcrcTokenOptions, crate::icrc::IcrcCommandError> {
    IcrcTokenOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
}

pub(in crate::icrc) fn parse_capabilities_options(args: &[&str]) -> IcrcCapabilitiesOptions {
    try_parse_capabilities_options(args).expect("parse ICRC capabilities options")
}

pub(in crate::icrc) fn try_parse_capabilities_options(
    args: &[&str],
) -> Result<IcrcCapabilitiesOptions, crate::icrc::IcrcCommandError> {
    IcrcCapabilitiesOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
}

pub(in crate::icrc) fn parse_balance_options(args: &[&str]) -> IcrcBalanceOptions {
    try_parse_balance_options(args).expect("parse ICRC balance options")
}

pub(in crate::icrc) fn try_parse_balance_options(
    args: &[&str],
) -> Result<IcrcBalanceOptions, crate::icrc::IcrcCommandError> {
    IcrcBalanceOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
}

pub(in crate::icrc) fn parse_allowance_options(args: &[&str]) -> IcrcAllowanceOptions {
    try_parse_allowance_options(args).expect("parse ICRC allowance options")
}

pub(in crate::icrc) fn try_parse_allowance_options(
    args: &[&str],
) -> Result<IcrcAllowanceOptions, crate::icrc::IcrcCommandError> {
    IcrcAllowanceOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
}

pub(in crate::icrc) fn parse_index_options(args: &[&str]) -> IcrcIndexOptions {
    try_parse_index_options(args).expect("parse ICRC index options")
}

pub(in crate::icrc) fn try_parse_index_options(
    args: &[&str],
) -> Result<IcrcIndexOptions, crate::icrc::IcrcCommandError> {
    IcrcIndexOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
}

pub(in crate::icrc) fn parse_transactions_options(args: &[&str]) -> IcrcTransactionsOptions {
    try_parse_transactions_options(args).expect("parse ICRC transactions options")
}

pub(in crate::icrc) fn try_parse_transactions_options(
    args: &[&str],
) -> Result<IcrcTransactionsOptions, crate::icrc::IcrcCommandError> {
    IcrcTransactionsOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
}

pub(in crate::icrc) fn parse_block_types_options(args: &[&str]) -> IcrcBlockTypesOptions {
    try_parse_block_types_options(args).expect("parse ICRC block types options")
}

pub(in crate::icrc) fn try_parse_block_types_options(
    args: &[&str],
) -> Result<IcrcBlockTypesOptions, crate::icrc::IcrcCommandError> {
    IcrcBlockTypesOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
}

pub(in crate::icrc) fn parse_archives_options(args: &[&str]) -> IcrcArchivesOptions {
    try_parse_archives_options(args).expect("parse ICRC archives options")
}

pub(in crate::icrc) fn try_parse_archives_options(
    args: &[&str],
) -> Result<IcrcArchivesOptions, crate::icrc::IcrcCommandError> {
    IcrcArchivesOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
}

pub(in crate::icrc) fn parse_tip_certificate_options(args: &[&str]) -> IcrcTipCertificateOptions {
    try_parse_tip_certificate_options(args).expect("parse ICRC tip certificate options")
}

pub(in crate::icrc) fn try_parse_tip_certificate_options(
    args: &[&str],
) -> Result<IcrcTipCertificateOptions, crate::icrc::IcrcCommandError> {
    IcrcTipCertificateOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
}

pub(in crate::icrc) fn root_usage() -> String {
    usage()
}

pub(in crate::icrc) fn token_usage() -> String {
    icrc_token_usage()
}

pub(in crate::icrc) fn capabilities_usage() -> String {
    icrc_capabilities_usage()
}

pub(in crate::icrc) fn balance_usage() -> String {
    icrc_balance_usage()
}

pub(in crate::icrc) fn allowance_usage() -> String {
    icrc_allowance_usage()
}

pub(in crate::icrc) fn index_usage() -> String {
    icrc_index_usage()
}

pub(in crate::icrc) fn transactions_usage() -> String {
    icrc_transactions_usage()
}

pub(in crate::icrc) fn block_types_usage() -> String {
    icrc_block_types_usage()
}

pub(in crate::icrc) fn archives_usage() -> String {
    icrc_archives_usage()
}

pub(in crate::icrc) fn tip_certificate_usage() -> String {
    icrc_tip_certificate_usage()
}
