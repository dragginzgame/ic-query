//! Module: icrc::commands::options
//!
//! Responsibility: parse Clap matches into typed ICRC command options.
//! Does not own: command dispatch, report construction, or output.
//! Boundary: validates command arguments before dispatch constructs public requests.

use super::{
    FOLLOW_ARCHIVES_ARG, FROM_CANISTER_ID_ARG, LEDGER_CANISTER_ID_ARG, LIMIT_ARG,
    OWNER_PRINCIPAL_ARG, OWNER_SUBACCOUNT_ARG, PRINCIPAL_ARG, SPENDER_PRINCIPAL_ARG,
    SPENDER_SUBACCOUNT_ARG, START_ARG, SUBACCOUNT_ARG, format_from_matches, icrc_allowance_command,
    icrc_allowance_usage, icrc_archives_command, icrc_archives_usage, icrc_balance_command,
    icrc_balance_usage, icrc_block_types_command, icrc_block_types_usage,
    icrc_capabilities_command, icrc_capabilities_usage, icrc_index_command, icrc_index_usage,
    icrc_tip_certificate_command, icrc_tip_certificate_usage, icrc_token_command, icrc_token_usage,
    icrc_transactions_command, icrc_transactions_usage, source_endpoint_from_matches,
};
use crate::{
    cli::{
        clap::{parse_matches_or_usage, required_string, required_typed, string_option},
        common::OutputFormat,
    },
    icrc::model::IcrcError,
};
use std::ffi::OsString;

///
/// IcrcCapabilitiesOptions
///
/// Clap-parsed options for generic ICRC capability probes.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcCapabilitiesOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcCapabilitiesOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches_or_usage(icrc_capabilities_command(), args, icrc_capabilities_usage)
                .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcTokenOptions
///
/// Clap-parsed options for generic ICRC token metadata queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcTokenOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcTokenOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_token_command(), args, icrc_token_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcBalanceOptions
///
/// Clap-parsed options for generic ICRC account balance queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcBalanceOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) account_owner: String,
    pub(in crate::icrc) subaccount_hex: Option<String>,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcBalanceOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_balance_command(), args, icrc_balance_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            account_owner: required_string(&matches, PRINCIPAL_ARG),
            subaccount_hex: string_option(&matches, SUBACCOUNT_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcAllowanceOptions
///
/// Clap-parsed options for generic ICRC allowance queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAllowanceOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) account_owner: String,
    pub(in crate::icrc) account_subaccount_hex: Option<String>,
    pub(in crate::icrc) spender_owner: String,
    pub(in crate::icrc) spender_subaccount_hex: Option<String>,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcAllowanceOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_allowance_command(), args, icrc_allowance_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            account_owner: required_string(&matches, OWNER_PRINCIPAL_ARG),
            account_subaccount_hex: string_option(&matches, OWNER_SUBACCOUNT_ARG),
            spender_owner: required_string(&matches, SPENDER_PRINCIPAL_ARG),
            spender_subaccount_hex: string_option(&matches, SPENDER_SUBACCOUNT_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcIndexOptions
///
/// Clap-parsed options for generic ICRC index discovery queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcIndexOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcIndexOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_index_command(), args, icrc_index_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcTransactionsOptions
///
/// Clap-parsed options for generic ICRC transaction history queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcTransactionsOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) start: u64,
    pub(in crate::icrc) limit: u32,
    pub(in crate::icrc) follow_archives: bool,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcTransactionsOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches_or_usage(icrc_transactions_command(), args, icrc_transactions_usage)
                .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            start: required_typed(&matches, START_ARG),
            limit: required_typed(&matches, LIMIT_ARG),
            follow_archives: matches.get_flag(FOLLOW_ARCHIVES_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcBlockTypesOptions
///
/// Clap-parsed options for generic ICRC-3 supported block type queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcBlockTypesOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcBlockTypesOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches_or_usage(icrc_block_types_command(), args, icrc_block_types_usage)
                .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcArchivesOptions
///
/// Clap-parsed options for generic ICRC-3 archive range queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcArchivesOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) from_canister_id: Option<String>,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcArchivesOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_archives_command(), args, icrc_archives_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            from_canister_id: string_option(&matches, FROM_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcTipCertificateOptions
///
/// Clap-parsed options for generic ICRC-3 tip certificate queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcTipCertificateOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcTipCertificateOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(
            icrc_tip_certificate_command(),
            args,
            icrc_tip_certificate_usage,
        )
        .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}
