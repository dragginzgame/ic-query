//! Module: icrc::commands::options
//!
//! Responsibility: parse Clap matches into ICRC command options and public requests.
//! Does not own: command dispatch, report construction, or output.
//! Boundary: carries parsed values into requests without report construction or IO.

use super::sort::{IcrcAccountSortArg, IcrcAccountTransactionSortArg, IcrcHolderSortArg};
use super::{
    ACCOUNT_ID_ARG, AFTER_ARG, BEFORE_ARG, END_ARG, FOLLOW_ARCHIVES_ARG, FROM_CANISTER_ID_ARG,
    INDEX_CANISTER_ID_ARG, LEDGER_CANISTER_ID_ARG, LIMIT_ARG, MAX_PAGES_ARG, OWNER_ARG,
    OWNER_PRINCIPAL_ARG, OWNER_SUBACCOUNT_ARG, PAGE_SIZE_ARG, PRINCIPAL_ARG, SORT_ARG, SORT_BY_ARG,
    SPENDER_PRINCIPAL_ARG, SPENDER_SUBACCOUNT_ARG, START_ARG, STEP_ARG, SUBACCOUNT_ARG,
};
use crate::cli::{
    clap::{required_string, required_typed, string_option, typed_option},
    common::{OutputFormat, SourceOutputOptions, resolve_time_bounds},
};
use clap::ArgMatches;
use ic_query::{
    ic::{
        DEFAULT_ICRC_TOKEN_VALUE_WINDOW_SECS, DEFAULT_ICRC_TOTAL_SUPPLY_WINDOW_SECS,
        IcIcrcAccountInfoRequest, IcIcrcAccountListQuery, IcIcrcAccountListRequest,
        IcIcrcHolderListQuery, IcIcrcHolderListRequest, IcIcrcTokenValueQuery,
        IcIcrcTokenValueRequest, IcIcrcTotalSupplyQuery, IcIcrcTotalSupplyRequest,
        MIN_ICRC_ANALYTICS_TIMESTAMP,
    },
    icrc::{
        IcrcAccountTransactionCacheRequest, IcrcAccountTransactionListRequest,
        IcrcAccountTransactionPageRequest, IcrcAccountTransactionRefreshRequest,
        IcrcAllowanceRequest, IcrcArchivesRequest, IcrcTransactionsRequest,
    },
};

pub(in crate::icrc) fn analytics_account_list_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcIcrcAccountListRequest) {
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    let mut query = IcIcrcAccountListQuery::new(
        required_typed(matches, LIMIT_ARG),
        required_typed::<IcrcAccountSortArg>(matches, SORT_BY_ARG).into(),
    );
    query.owner = string_option(matches, OWNER_ARG);
    query.after = string_option(matches, AFTER_ARG);
    query.before = string_option(matches, BEFORE_ARG);
    (
        format,
        IcIcrcAccountListRequest::new(dashboard, ledger, query),
    )
}

pub(in crate::icrc) fn analytics_account_info_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcIcrcAccountInfoRequest) {
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    (
        format,
        IcIcrcAccountInfoRequest::new(dashboard, ledger, required_string(matches, ACCOUNT_ID_ARG)),
    )
}

pub(in crate::icrc) fn analytics_holder_list_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcIcrcHolderListRequest) {
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    let mut query = IcIcrcHolderListQuery::new(
        required_typed(matches, LIMIT_ARG),
        required_typed::<IcrcHolderSortArg>(matches, SORT_BY_ARG).into(),
    );
    query.after = string_option(matches, AFTER_ARG);
    query.before = string_option(matches, BEFORE_ARG);
    (
        format,
        IcIcrcHolderListRequest::new(dashboard, ledger, query),
    )
}

pub(in crate::icrc) fn analytics_token_value_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcIcrcTokenValueRequest) {
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    let (start, end) = analytics_time_bounds(
        matches,
        now_unix_secs,
        DEFAULT_ICRC_TOKEN_VALUE_WINDOW_SECS,
        0,
    );
    let query = IcIcrcTokenValueQuery::new(start, end, required_typed(matches, LIMIT_ARG));
    (
        format,
        IcIcrcTokenValueRequest::new(dashboard, ledger, query),
    )
}

pub(in crate::icrc) fn analytics_total_supply_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcIcrcTotalSupplyRequest) {
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    let (start, end) = analytics_time_bounds(
        matches,
        now_unix_secs,
        DEFAULT_ICRC_TOTAL_SUPPLY_WINDOW_SECS,
        MIN_ICRC_ANALYTICS_TIMESTAMP,
    );
    let query = IcIcrcTotalSupplyQuery::new(start, end, required_typed(matches, STEP_ARG));
    (
        format,
        IcIcrcTotalSupplyRequest::new(dashboard, ledger, query),
    )
}

fn analytics_time_bounds(
    matches: &ArgMatches,
    now_unix_secs: u64,
    default_window_secs: u64,
    minimum_default_start: u64,
) -> (u64, u64) {
    resolve_time_bounds(
        typed_option(matches, START_ARG),
        typed_option(matches, END_ARG),
        now_unix_secs,
        default_window_secs,
        minimum_default_start,
    )
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
        let ledger_canister_id = required_string(matches, LEDGER_CANISTER_ID_ARG);
        let source = SourceOutputOptions::from_matches(matches);
        Self {
            ledger_canister_id,
            format: source.format,
            source_endpoint: source.source_endpoint,
        }
    }

    pub(in crate::icrc) fn into_dashboard_request(
        self,
        now_unix_secs: u64,
    ) -> (OutputFormat, ic_query::ic::IcDashboardRequest, String) {
        let (format, request) = SourceOutputOptions {
            format: self.format,
            source_endpoint: self.source_endpoint,
        }
        .into_dashboard_request(now_unix_secs);
        (format, request, self.ledger_canister_id)
    }

    pub(in crate::icrc) fn into_ledger_request(
        self,
        now_unix_secs: u64,
    ) -> (OutputFormat, ic_query::icrc::IcrcLedgerRequest) {
        (
            self.format,
            ic_query::icrc::IcrcLedgerRequest::new(
                self.source_endpoint,
                now_unix_secs,
                self.ledger_canister_id,
            ),
        )
    }
}

///
/// IcrcAccountOptions
///
/// Shared ledger, output, endpoint and account identity for ICRC account commands.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAccountOptions {
    pub(in crate::icrc) ledger: IcrcLedgerOptions,
    pub(in crate::icrc) account_owner: String,
    pub(in crate::icrc) subaccount_hex: Option<String>,
}

impl IcrcAccountOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            ledger: IcrcLedgerOptions::from_matches(matches),
            account_owner: required_string(matches, PRINCIPAL_ARG),
            subaccount_hex: string_option(matches, SUBACCOUNT_ARG),
        }
    }

    pub(in crate::icrc) fn into_cache_request(
        self,
        cache_root: std::path::PathBuf,
    ) -> (OutputFormat, IcrcAccountTransactionCacheRequest) {
        (
            self.ledger.format,
            IcrcAccountTransactionCacheRequest {
                cache_root,
                source_endpoint: self.ledger.source_endpoint,
                ledger_canister_id: self.ledger.ledger_canister_id,
                account_owner: self.account_owner,
                subaccount_hex: self.subaccount_hex,
            },
        )
    }
}

pub(in crate::icrc) fn allowance_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcAllowanceRequest) {
    let (format, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_ledger_request(now_unix_secs);
    (
        format,
        IcrcAllowanceRequest {
            ledger,
            account_owner: required_string(matches, OWNER_PRINCIPAL_ARG),
            account_subaccount_hex: string_option(matches, OWNER_SUBACCOUNT_ARG),
            spender_owner: required_string(matches, SPENDER_PRINCIPAL_ARG),
            spender_subaccount_hex: string_option(matches, SPENDER_SUBACCOUNT_ARG),
        },
    )
}

pub(in crate::icrc) fn account_transaction_page_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcAccountTransactionPageRequest) {
    let target = IcrcAccountOptions::from_matches(matches);
    let (format, ledger) = target.ledger.into_ledger_request(now_unix_secs);
    (
        format,
        IcrcAccountTransactionPageRequest {
            ledger,
            index_canister_id: string_option(matches, INDEX_CANISTER_ID_ARG),
            account_owner: target.account_owner,
            subaccount_hex: target.subaccount_hex,
            start: string_option(matches, START_ARG),
            limit: required_typed(matches, LIMIT_ARG),
        },
    )
}

pub(in crate::icrc) fn account_transaction_list_request(
    matches: &ArgMatches,
    cache_root: std::path::PathBuf,
) -> (OutputFormat, IcrcAccountTransactionListRequest) {
    let (format, cache) = IcrcAccountOptions::from_matches(matches).into_cache_request(cache_root);
    (
        format,
        IcrcAccountTransactionListRequest {
            cache,
            limit: required_typed(matches, LIMIT_ARG),
            sort: required_typed::<IcrcAccountTransactionSortArg>(matches, SORT_ARG).into(),
        },
    )
}

pub(in crate::icrc) fn account_transaction_refresh_request(
    matches: &ArgMatches,
    cache_root: std::path::PathBuf,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcAccountTransactionRefreshRequest) {
    let (format, cache) = IcrcAccountOptions::from_matches(matches).into_cache_request(cache_root);
    (
        format,
        IcrcAccountTransactionRefreshRequest {
            cache,
            now_unix_secs,
            index_canister_id: string_option(matches, INDEX_CANISTER_ID_ARG),
            page_size: required_typed(matches, PAGE_SIZE_ARG),
            max_pages: typed_option(matches, MAX_PAGES_ARG),
        },
    )
}

pub(in crate::icrc) fn transactions_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcTransactionsRequest) {
    let (format, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_ledger_request(now_unix_secs);
    (
        format,
        IcrcTransactionsRequest {
            ledger,
            start: required_typed(matches, START_ARG),
            limit: required_typed(matches, LIMIT_ARG),
            follow_archives: matches.get_flag(FOLLOW_ARCHIVES_ARG),
        },
    )
}

pub(in crate::icrc) fn archives_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcArchivesRequest) {
    let (format, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_ledger_request(now_unix_secs);
    (
        format,
        IcrcArchivesRequest {
            ledger,
            from_canister_id: string_option(matches, FROM_CANISTER_ID_ARG),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> IcrcLedgerOptions {
        IcrcLedgerOptions {
            ledger_canister_id: "aaaaa-aa".to_string(),
            source_endpoint: "https://example.com/api".to_string(),
            format: OutputFormat::Json,
        }
    }

    #[test]
    fn native_ledger_request_retains_explicit_target_time_and_output() {
        let (format, request) = target().into_ledger_request(123);
        assert_eq!(format, OutputFormat::Json);
        assert_eq!(request.source_endpoint, "https://example.com/api");
        assert_eq!(request.ledger_canister_id, "aaaaa-aa");
        assert_eq!(request.now_unix_secs, 123);
    }
}
