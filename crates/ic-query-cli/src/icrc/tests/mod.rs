use super::commands::{
    IcrcAccountTransactionCacheOptions, IcrcAccountTransactionListOptions,
    IcrcAccountTransactionPageOptions, IcrcAccountTransactionRefreshOptions, IcrcAllowanceOptions,
    IcrcArchivesOptions, IcrcBalanceOptions, IcrcLedgerOptions, IcrcTransactionsOptions,
    command as icrc_command, icrc_account_command, icrc_account_transaction_cache_command,
    icrc_account_transaction_cache_status_command, icrc_account_transaction_command,
    icrc_account_transaction_list_command, icrc_account_transaction_page_command,
    icrc_account_transaction_refresh_command, icrc_allowance_command, icrc_archives_command,
    icrc_balance_command, icrc_block_types_command, icrc_capabilities_command, icrc_index_command,
    icrc_ledger_command, icrc_tip_certificate_command, icrc_token_command,
    icrc_transactions_command,
};
use crate::cli::{
    clap::{parse_matches_or_usage, render_help},
    common::OutputFormat,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::icrc::IcrcAccountTransactionSort;

const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const INDEX_CANISTER_ID: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const ACCOUNT_OWNER: &str = "aaaaa-aa";
const SOURCE_ENDPOINT: &str = "https://icp-api.io";
const SUBACCOUNT: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

fn parse_test_options<Options>(
    command: ClapCommand,
    args: &[&str],
    from_matches: fn(&ArgMatches) -> Options,
) -> Options {
    let matches =
        parse_matches_or_usage(command, args.iter().copied().map(std::ffi::OsString::from))
            .expect("parse ICRC test options");
    from_matches(&matches)
}

#[test]
fn token_options_parse_through_clap() {
    let options = parse_test_options(
        icrc_token_command(),
        &[
            LEDGER_CANISTER_ID,
            "--json",
            "--source-endpoint",
            SOURCE_ENDPOINT,
        ],
        IcrcLedgerOptions::from_matches,
    );

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, SOURCE_ENDPOINT);
}

#[test]
fn capabilities_options_parse_through_clap() {
    let options = parse_test_options(
        icrc_capabilities_command(),
        &[LEDGER_CANISTER_ID, "--json"],
        IcrcLedgerOptions::from_matches,
    );

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn balance_options_parse_through_clap_and_normalize_subaccount() {
    let options = parse_test_options(
        icrc_balance_command(),
        &[
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            "--subaccount",
            SUBACCOUNT,
            "--json",
        ],
        IcrcBalanceOptions::from_matches,
    );

    assert_eq!(options.ledger.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.subaccount_hex.as_deref(), Some(SUBACCOUNT));
    assert_eq!(options.ledger.format, OutputFormat::Json);
}

#[test]
fn allowance_options_parse_through_clap_and_normalize_subaccounts() {
    let options = parse_test_options(
        icrc_allowance_command(),
        &[
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            ACCOUNT_OWNER,
            "--owner-subaccount",
            SUBACCOUNT,
            "--spender-subaccount",
            SUBACCOUNT,
        ],
        IcrcAllowanceOptions::from_matches,
    );

    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.spender_owner, ACCOUNT_OWNER);
    assert_eq!(options.account_subaccount_hex.as_deref(), Some(SUBACCOUNT));
    assert_eq!(options.spender_subaccount_hex.as_deref(), Some(SUBACCOUNT));
}

#[test]
fn account_transaction_page_options_parse_arbitrary_nat_cursor() {
    let options = parse_test_options(
        icrc_account_transaction_page_command(),
        &[
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            "--index-canister-id",
            INDEX_CANISTER_ID,
            "--subaccount",
            SUBACCOUNT,
            "--start",
            "18446744073709551616",
            "--limit",
            "42",
            "--json",
            "--source-endpoint",
            SOURCE_ENDPOINT,
        ],
        IcrcAccountTransactionPageOptions::from_matches,
    );

    assert_eq!(options.target.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(
        options.index_canister_id.as_deref(),
        Some(INDEX_CANISTER_ID)
    );
    assert_eq!(options.target.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.target.subaccount_hex.as_deref(), Some(SUBACCOUNT));
    assert_eq!(options.start.as_deref(), Some("18446744073709551616"));
    assert_eq!(options.limit, 42);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.target.source_endpoint, SOURCE_ENDPOINT);
}

#[test]
fn account_transaction_list_options_parse_cache_view() {
    let options = parse_test_options(
        icrc_account_transaction_list_command(),
        &[
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            "--limit",
            "250",
            "--sort",
            "oldest",
        ],
        IcrcAccountTransactionListOptions::from_matches,
    );

    assert_eq!(options.limit, 250);
    assert_eq!(options.sort, IcrcAccountTransactionSort::Oldest);
}

#[test]
fn account_transaction_refresh_options_parse_collection_bounds() {
    let options = parse_test_options(
        icrc_account_transaction_refresh_command(),
        &[
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            "--index-canister-id",
            INDEX_CANISTER_ID,
            "--page-size",
            "100",
            "--max-pages",
            "20",
        ],
        IcrcAccountTransactionRefreshOptions::from_matches,
    );

    assert_eq!(options.page_size, 100);
    assert_eq!(options.max_pages, Some(20));
    assert_eq!(
        options.index_canister_id.as_deref(),
        Some(INDEX_CANISTER_ID)
    );
}

#[test]
fn account_transaction_cache_options_parse_identity() {
    let options = parse_test_options(
        icrc_account_transaction_cache_status_command(),
        &[
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            "--subaccount",
            SUBACCOUNT,
        ],
        IcrcAccountTransactionCacheOptions::from_matches,
    );

    assert_eq!(options.target.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.target.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.target.subaccount_hex.as_deref(), Some(SUBACCOUNT));
}

#[test]
fn index_options_parse_through_clap() {
    let options = parse_test_options(
        icrc_index_command(),
        &[LEDGER_CANISTER_ID, "--json"],
        IcrcLedgerOptions::from_matches,
    );

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn transactions_options_parse_through_clap() {
    let options = parse_test_options(
        icrc_transactions_command(),
        &[
            LEDGER_CANISTER_ID,
            "--start",
            "17",
            "--limit",
            "42",
            "--follow-archives",
        ],
        IcrcTransactionsOptions::from_matches,
    );

    assert_eq!(options.ledger.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.start, 17);
    assert_eq!(options.limit, 42);
    assert!(options.follow_archives);
}

#[test]
fn block_types_options_parse_through_clap() {
    let options = parse_test_options(
        icrc_block_types_command(),
        &[LEDGER_CANISTER_ID, "--json"],
        IcrcLedgerOptions::from_matches,
    );

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn archives_options_parse_through_clap() {
    let options = parse_test_options(
        icrc_archives_command(),
        &[LEDGER_CANISTER_ID, "--from", INDEX_CANISTER_ID, "--json"],
        IcrcArchivesOptions::from_matches,
    );

    assert_eq!(options.ledger.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.from_canister_id.as_deref(), Some(INDEX_CANISTER_ID));
    assert_eq!(options.ledger.format, OutputFormat::Json);
}

#[test]
fn tip_certificate_options_parse_through_clap() {
    let options = parse_test_options(
        icrc_tip_certificate_command(),
        &[LEDGER_CANISTER_ID, "--json"],
        IcrcLedgerOptions::from_matches,
    );

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn usage_mentions_icrc_command_surface() {
    let root = render_help(icrc_command());
    let ledger = render_help(icrc_ledger_command());
    let account = render_help(icrc_account_command());
    let token = render_help(icrc_token_command());
    let capabilities = render_help(icrc_capabilities_command());
    let balance = render_help(icrc_balance_command());
    let allowance = render_help(icrc_allowance_command());
    let account_transaction = render_help(icrc_account_transaction_command());
    let account_transaction_page = render_help(icrc_account_transaction_page_command());
    let account_transaction_list = render_help(icrc_account_transaction_list_command());
    let account_transaction_refresh = render_help(icrc_account_transaction_refresh_command());
    let account_transaction_cache = render_help(icrc_account_transaction_cache_command());
    let account_transaction_cache_status =
        render_help(icrc_account_transaction_cache_status_command());
    let index = render_help(icrc_index_command());
    let transactions = render_help(icrc_transactions_command());
    let block_types = render_help(icrc_block_types_command());
    let archives = render_help(icrc_archives_command());
    let tip_certificate = render_help(icrc_tip_certificate_command());

    for (usage, needle) in [
        (root.as_str(), "ledger"),
        (root.as_str(), "account"),
        (ledger.as_str(), "capabilities"),
        (ledger.as_str(), "transactions"),
        (account.as_str(), "balance"),
        (account.as_str(), "allowance"),
        (account.as_str(), "transaction"),
        (token.as_str(), "ledger-canister-id"),
        (capabilities.as_str(), "ledger-canister-id"),
        (balance.as_str(), "principal"),
        (allowance.as_str(), "spender-principal"),
        (account_transaction.as_str(), "refresh"),
        (account_transaction_page.as_str(), "--index-canister-id"),
        (account_transaction_list.as_str(), "--sort"),
        (account_transaction_refresh.as_str(), "--page-size"),
        (account_transaction_cache.as_str(), "status"),
        (
            account_transaction_cache_status.as_str(),
            "ledger-canister-id",
        ),
        (index.as_str(), "ledger-canister-id"),
        (transactions.as_str(), "follow-archives"),
        (block_types.as_str(), "ledger-canister-id"),
        (archives.as_str(), "--from"),
        (tip_certificate.as_str(), "ledger-canister-id"),
    ] {
        assert!(usage.contains(needle), "missing {needle:?} in {usage}");
    }

    assert!(token.contains("icq icrc ledger token"));
    assert!(balance.contains("icq icrc account balance"));
    assert!(account_transaction.contains("icq icrc account transaction"));
    assert!(account_transaction_page.contains("icq icrc account transaction page"));

    for usage in [
        token,
        capabilities,
        balance,
        allowance,
        account_transaction_page,
        index,
        transactions,
        block_types,
        archives,
        tip_certificate,
    ] {
        assert!(usage.contains("Collection mode: Live query"));
    }

    assert!(account_transaction_list.contains("Collection mode: Local cache inspection"));
    assert!(account_transaction_cache_status.contains("Collection mode: Local cache inspection"));
    assert!(account_transaction_refresh.contains("Collection mode: Forced live refresh"));
}
