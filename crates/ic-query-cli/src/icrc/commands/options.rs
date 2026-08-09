//! Module: icrc::commands::options
//!
//! Responsibility: parse Clap matches into typed ICRC command options.
//! Does not own: command dispatch, report construction, or output.
//! Boundary: validates command arguments before dispatch constructs public requests.

use super::{
    ACCOUNT_ID_ARG, AFTER_ARG, BEFORE_ARG, END_ARG, FOLLOW_ARCHIVES_ARG, FROM_CANISTER_ID_ARG,
    INDEX_CANISTER_ID_ARG, LEDGER_CANISTER_ID_ARG, LIMIT_ARG, MAX_PAGES_ARG, OWNER_ARG,
    OWNER_PRINCIPAL_ARG, OWNER_SUBACCOUNT_ARG, PAGE_SIZE_ARG, PRINCIPAL_ARG, SORT_ARG, SORT_BY_ARG,
    SPENDER_PRINCIPAL_ARG, SPENDER_SUBACCOUNT_ARG, START_ARG, STEP_ARG, SUBACCOUNT_ARG,
    format_from_matches, source_endpoint_from_matches,
};
use crate::cli::{
    clap::{required_string, required_typed, string_option, typed_option},
    common::OutputFormat,
};
use clap::ArgMatches;
use ic_query::{
    ic::{DEFAULT_ICRC_TOTAL_SUPPLY_STEP_SECS, IcIcrcAccountSort, IcIcrcHolderSort},
    icrc::IcrcAccountTransactionSort,
};

///
/// IcrcAnalyticsAccountListOptions
///
/// Clap-parsed bounds and sort for one official account-index page.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAnalyticsAccountListOptions {
    pub(in crate::icrc) target: IcrcLedgerOptions,
    pub(in crate::icrc) owner: Option<String>,
    pub(in crate::icrc) after: Option<String>,
    pub(in crate::icrc) before: Option<String>,
    pub(in crate::icrc) limit: u16,
    pub(in crate::icrc) sort_by: IcIcrcAccountSort,
}

impl IcrcAnalyticsAccountListOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            target: IcrcLedgerOptions::from_matches(matches),
            owner: string_option(matches, OWNER_ARG),
            after: string_option(matches, AFTER_ARG),
            before: string_option(matches, BEFORE_ARG),
            limit: required_typed(matches, LIMIT_ARG),
            sort_by: match required_string(matches, SORT_BY_ARG).as_str() {
                "id" => IcIcrcAccountSort::Id,
                "-id" => IcIcrcAccountSort::IdDescending,
                "balance" => IcIcrcAccountSort::Balance,
                "-balance" => IcIcrcAccountSort::BalanceDescending,
                "total_transactions" => IcIcrcAccountSort::TotalTransactions,
                "-total_transactions" => IcIcrcAccountSort::TotalTransactionsDescending,
                "created_timestamp" => IcIcrcAccountSort::CreatedTimestamp,
                "-created_timestamp" => IcIcrcAccountSort::CreatedTimestampDescending,
                "owner" => IcIcrcAccountSort::Owner,
                "-owner" => IcIcrcAccountSort::OwnerDescending,
                _ => unreachable!("Clap restricts ICRC account sort values"),
            },
        }
    }
}

///
/// IcrcAnalyticsAccountInfoOptions
///
/// Clap-parsed target for one exact official account-index lookup.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAnalyticsAccountInfoOptions {
    pub(in crate::icrc) target: IcrcLedgerOptions,
    pub(in crate::icrc) account_id: String,
}

impl IcrcAnalyticsAccountInfoOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            target: IcrcLedgerOptions::from_matches(matches),
            account_id: required_string(matches, ACCOUNT_ID_ARG),
        }
    }
}

///
/// IcrcAnalyticsHolderListOptions
///
/// Clap-parsed bounds and sort for one official holder-index page.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAnalyticsHolderListOptions {
    pub(in crate::icrc) target: IcrcLedgerOptions,
    pub(in crate::icrc) after: Option<String>,
    pub(in crate::icrc) before: Option<String>,
    pub(in crate::icrc) limit: u16,
    pub(in crate::icrc) sort_by: IcIcrcHolderSort,
}

impl IcrcAnalyticsHolderListOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            target: IcrcLedgerOptions::from_matches(matches),
            after: string_option(matches, AFTER_ARG),
            before: string_option(matches, BEFORE_ARG),
            limit: required_typed(matches, LIMIT_ARG),
            sort_by: match required_string(matches, SORT_BY_ARG).as_str() {
                "balance" => IcIcrcHolderSort::Balance,
                "-balance" => IcIcrcHolderSort::BalanceDescending,
                "total_transactions" => IcIcrcHolderSort::TotalTransactions,
                "-total_transactions" => IcIcrcHolderSort::TotalTransactionsDescending,
                "created_timestamp" => IcIcrcHolderSort::CreatedTimestamp,
                "-created_timestamp" => IcIcrcHolderSort::CreatedTimestampDescending,
                "principal" => IcIcrcHolderSort::Principal,
                "-principal" => IcIcrcHolderSort::PrincipalDescending,
                _ => unreachable!("Clap restricts ICRC holder sort values"),
            },
        }
    }
}

///
/// IcrcAnalyticsWindowOptions
///
/// Shared ledger target and optional time bounds for official ICRC analytics.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAnalyticsWindowOptions {
    pub(in crate::icrc) target: IcrcLedgerOptions,
    pub(in crate::icrc) start_unix_secs: Option<u64>,
    pub(in crate::icrc) end_unix_secs: Option<u64>,
}

impl IcrcAnalyticsWindowOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            target: IcrcLedgerOptions::from_matches(matches),
            start_unix_secs: typed_option(matches, START_ARG),
            end_unix_secs: typed_option(matches, END_ARG),
        }
    }
}

///
/// IcrcAnalyticsTotalSupplyOptions
///
/// Clap-parsed bounds plus the shared target for one official total-supply series.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAnalyticsTotalSupplyOptions {
    pub(in crate::icrc) window: IcrcAnalyticsWindowOptions,
    pub(in crate::icrc) step_secs: u32,
}

impl IcrcAnalyticsTotalSupplyOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        let step_secs =
            string_option(matches, STEP_ARG).map_or(DEFAULT_ICRC_TOTAL_SUPPLY_STEP_SECS, |value| {
                value
                    .parse()
                    .expect("clap restricts ICRC analytics step values")
            });
        Self {
            window: IcrcAnalyticsWindowOptions::from_matches(matches),
            step_secs,
        }
    }
}

///
/// IcrcAnalyticsTokenValueOptions
///
/// Clap-parsed time and row bounds for one official token-value series.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAnalyticsTokenValueOptions {
    pub(in crate::icrc) window: IcrcAnalyticsWindowOptions,
    pub(in crate::icrc) limit: u16,
}

impl IcrcAnalyticsTokenValueOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            window: IcrcAnalyticsWindowOptions::from_matches(matches),
            limit: required_typed(matches, LIMIT_ARG),
        }
    }
}

///
/// IcrcLedgerOptions
///
/// Shared ledger target, output, and endpoint options for live ICRC queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcLedgerOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcLedgerOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            ledger_canister_id: required_string(matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(matches),
            source_endpoint: source_endpoint_from_matches(matches),
        }
    }
}

///
/// IcrcBalanceOptions
///
/// Clap-parsed options for generic ICRC account balance queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcBalanceOptions {
    pub(in crate::icrc) ledger: IcrcLedgerOptions,
    pub(in crate::icrc) account_owner: String,
    pub(in crate::icrc) subaccount_hex: Option<String>,
}

impl IcrcBalanceOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            ledger: IcrcLedgerOptions::from_matches(matches),
            account_owner: required_string(matches, PRINCIPAL_ARG),
            subaccount_hex: string_option(matches, SUBACCOUNT_ARG),
        }
    }
}

///
/// IcrcAllowanceOptions
///
/// Clap-parsed options for generic ICRC allowance queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAllowanceOptions {
    pub(in crate::icrc) ledger: IcrcLedgerOptions,
    pub(in crate::icrc) account_owner: String,
    pub(in crate::icrc) account_subaccount_hex: Option<String>,
    pub(in crate::icrc) spender_owner: String,
    pub(in crate::icrc) spender_subaccount_hex: Option<String>,
}

impl IcrcAllowanceOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            ledger: IcrcLedgerOptions::from_matches(matches),
            account_owner: required_string(matches, OWNER_PRINCIPAL_ARG),
            account_subaccount_hex: string_option(matches, OWNER_SUBACCOUNT_ARG),
            spender_owner: required_string(matches, SPENDER_PRINCIPAL_ARG),
            spender_subaccount_hex: string_option(matches, SPENDER_SUBACCOUNT_ARG),
        }
    }
}

///
/// IcrcAccountTargetOptions
///
/// Shared parsed ledger, account, subaccount, and endpoint identity for account-history commands.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAccountTargetOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) account_owner: String,
    pub(in crate::icrc) subaccount_hex: Option<String>,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcAccountTargetOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            ledger_canister_id: required_string(matches, LEDGER_CANISTER_ID_ARG),
            account_owner: required_string(matches, PRINCIPAL_ARG),
            subaccount_hex: string_option(matches, SUBACCOUNT_ARG),
            source_endpoint: source_endpoint_from_matches(matches),
        }
    }
}

///
/// IcrcAccountTransactionPageOptions
///
/// Clap-parsed options for one live ICRC index account-history page.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAccountTransactionPageOptions {
    pub(in crate::icrc) target: IcrcAccountTargetOptions,
    pub(in crate::icrc) index_canister_id: Option<String>,
    pub(in crate::icrc) start: Option<String>,
    pub(in crate::icrc) limit: u32,
    pub(in crate::icrc) format: OutputFormat,
}

impl IcrcAccountTransactionPageOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            target: IcrcAccountTargetOptions::from_matches(matches),
            index_canister_id: string_option(matches, INDEX_CANISTER_ID_ARG),
            start: string_option(matches, START_ARG),
            limit: required_typed(matches, LIMIT_ARG),
            format: format_from_matches(matches),
        }
    }
}

///
/// IcrcAccountTransactionListOptions
///
/// Clap-parsed options for a cache-only ICRC account transaction list.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAccountTransactionListOptions {
    pub(in crate::icrc) target: IcrcAccountTargetOptions,
    pub(in crate::icrc) limit: u32,
    pub(in crate::icrc) sort: IcrcAccountTransactionSort,
    pub(in crate::icrc) format: OutputFormat,
}

impl IcrcAccountTransactionListOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        let sort = match required_string(matches, SORT_ARG).as_str() {
            "newest" => IcrcAccountTransactionSort::Newest,
            "oldest" => IcrcAccountTransactionSort::Oldest,
            _ => unreachable!("Clap restricts account transaction sort values"),
        };
        Self {
            target: IcrcAccountTargetOptions::from_matches(matches),
            limit: required_typed(matches, LIMIT_ARG),
            sort,
            format: format_from_matches(matches),
        }
    }
}

///
/// IcrcAccountTransactionRefreshOptions
///
/// Clap-parsed options for a forced complete ICRC account-history refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAccountTransactionRefreshOptions {
    pub(in crate::icrc) target: IcrcAccountTargetOptions,
    pub(in crate::icrc) index_canister_id: Option<String>,
    pub(in crate::icrc) page_size: u32,
    pub(in crate::icrc) max_pages: Option<u32>,
    pub(in crate::icrc) format: OutputFormat,
}

impl IcrcAccountTransactionRefreshOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            target: IcrcAccountTargetOptions::from_matches(matches),
            index_canister_id: string_option(matches, INDEX_CANISTER_ID_ARG),
            page_size: required_typed(matches, PAGE_SIZE_ARG),
            max_pages: typed_option(matches, MAX_PAGES_ARG),
            format: format_from_matches(matches),
        }
    }
}

///
/// IcrcAccountTransactionCacheOptions
///
/// Clap-parsed options identifying one local ICRC account-history cache.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAccountTransactionCacheOptions {
    pub(in crate::icrc) target: IcrcAccountTargetOptions,
    pub(in crate::icrc) format: OutputFormat,
}

impl IcrcAccountTransactionCacheOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            target: IcrcAccountTargetOptions::from_matches(matches),
            format: format_from_matches(matches),
        }
    }
}

///
/// IcrcTransactionsOptions
///
/// Clap-parsed options for generic ICRC transaction history queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcTransactionsOptions {
    pub(in crate::icrc) ledger: IcrcLedgerOptions,
    pub(in crate::icrc) start: u64,
    pub(in crate::icrc) limit: u32,
    pub(in crate::icrc) follow_archives: bool,
}

impl IcrcTransactionsOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            ledger: IcrcLedgerOptions::from_matches(matches),
            start: required_typed(matches, START_ARG),
            limit: required_typed(matches, LIMIT_ARG),
            follow_archives: matches.get_flag(FOLLOW_ARCHIVES_ARG),
        }
    }
}

///
/// IcrcArchivesOptions
///
/// Clap-parsed options for generic ICRC-3 archive range queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcArchivesOptions {
    pub(in crate::icrc) ledger: IcrcLedgerOptions,
    pub(in crate::icrc) from_canister_id: Option<String>,
}

impl IcrcArchivesOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            ledger: IcrcLedgerOptions::from_matches(matches),
            from_canister_id: string_option(matches, FROM_CANISTER_ID_ARG),
        }
    }
}
