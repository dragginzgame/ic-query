use super::commands::options::{
    account_transaction_list_request, account_transaction_page_request,
    account_transaction_refresh_request, allowance_request, analytics_account_info_request,
    analytics_account_list_request, analytics_holder_list_request, analytics_token_value_request,
    analytics_total_supply_request, archives_request, transactions_request,
};
use super::commands::{
    IcrcAccountOptions, IcrcLedgerOptions, command as icrc_command, icrc_account_command,
    icrc_account_transaction_cache_command, icrc_account_transaction_cache_status_command,
    icrc_account_transaction_command, icrc_account_transaction_list_command,
    icrc_account_transaction_page_command, icrc_account_transaction_refresh_command,
    icrc_allowance_command, icrc_analytics_account_info_command,
    icrc_analytics_account_list_command, icrc_analytics_command,
    icrc_analytics_holder_list_command, icrc_analytics_token_values_command,
    icrc_analytics_total_supply_command, icrc_archives_command, icrc_balance_command,
    icrc_block_types_command, icrc_capabilities_command, icrc_index_command, icrc_ledger_command,
    icrc_tip_certificate_command, icrc_token_command, icrc_transactions_command,
};
use crate::cli::{
    clap::{parse_matches, render_help},
    common::OutputFormat,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::ic::{
    DEFAULT_ICRC_ACCOUNT_INFO_SOURCE_ENDPOINT, DEFAULT_ICRC_TOKEN_VALUE_LIMIT, IcIcrcAccountSort,
    IcIcrcHolderSort,
};
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
    let matches = crate::cli::parse_test_args(command, args).expect("parse ICRC test options");
    from_matches(&matches)
}

fn icrc_command_at(path: &[&str]) -> ClapCommand {
    let mut command = icrc_command();
    for name in path {
        command = command
            .find_subcommand(name)
            .expect("ICRC command path")
            .clone();
    }
    command
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
        IcrcAccountOptions::from_matches,
    );

    assert_eq!(options.ledger.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.subaccount_hex.as_deref(), Some(SUBACCOUNT));
    assert_eq!(options.ledger.format, OutputFormat::Json);
}

#[test]
fn allowance_options_parse_through_clap_and_normalize_subaccounts() {
    let (_, options) = parse_test_options(
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
        |matches| allowance_request(matches, 123),
    );

    assert_eq!(options.ledger.now_unix_secs, 123);
    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.spender_owner, ACCOUNT_OWNER);
    assert_eq!(options.account_subaccount_hex.as_deref(), Some(SUBACCOUNT));
    assert_eq!(options.spender_subaccount_hex.as_deref(), Some(SUBACCOUNT));
}

#[test]
fn account_transaction_page_options_parse_arbitrary_nat_cursor() {
    let (format, options) = parse_test_options(
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
        |matches| account_transaction_page_request(matches, 123),
    );

    assert_eq!(options.ledger.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(
        options.index_canister_id.as_deref(),
        Some(INDEX_CANISTER_ID)
    );
    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.subaccount_hex.as_deref(), Some(SUBACCOUNT));
    assert_eq!(options.start.as_deref(), Some("18446744073709551616"));
    assert_eq!(options.limit, 42);
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(options.ledger.source_endpoint, SOURCE_ENDPOINT);
}

#[test]
fn account_transaction_list_options_parse_cache_view() {
    let (_, options) = parse_test_options(
        icrc_account_transaction_list_command(),
        &[
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            "--limit",
            "250",
            "--sort",
            "oldest",
        ],
        |matches| account_transaction_list_request(matches, "/cache".into()),
    );

    assert_eq!(options.cache.cache_root, std::path::PathBuf::from("/cache"));
    assert_eq!(options.limit, 250);
    assert_eq!(options.sort, IcrcAccountTransactionSort::Oldest);
}

#[test]
fn account_transaction_refresh_options_parse_collection_bounds() {
    let (_, options) = parse_test_options(
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
        |matches| account_transaction_refresh_request(matches, "/cache".into(), 123),
    );

    assert_eq!(options.cache.cache_root, std::path::PathBuf::from("/cache"));
    assert_eq!(options.now_unix_secs, 123);
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
        IcrcAccountOptions::from_matches,
    );

    assert_eq!(options.ledger.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.subaccount_hex.as_deref(), Some(SUBACCOUNT));
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
    let (_, options) = parse_test_options(
        icrc_transactions_command(),
        &[
            LEDGER_CANISTER_ID,
            "--start",
            "17",
            "--limit",
            "42",
            "--follow-archives",
        ],
        |matches| transactions_request(matches, 123),
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
    let (format, options) = parse_test_options(
        icrc_archives_command(),
        &[LEDGER_CANISTER_ID, "--from", INDEX_CANISTER_ID, "--json"],
        |matches| archives_request(matches, 123),
    );

    assert_eq!(options.ledger.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.from_canister_id.as_deref(), Some(INDEX_CANISTER_ID));
    assert_eq!(format, OutputFormat::Json);
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
fn analytics_total_supply_options_parse_bounds_endpoint_and_format() {
    let (format, options) = parse_test_options(
        icrc_analytics_total_supply_command(),
        &[
            LEDGER_CANISTER_ID,
            "--start",
            "1785542400",
            "--end",
            "1785801600",
            "--step",
            "3600",
            "--json",
            "--source-endpoint",
            "https://example.com/api/v2",
        ],
        |matches| analytics_total_supply_request(matches, 1_785_801_600),
    );

    assert_eq!(options.analytics.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.query.start_unix_secs, 1_785_542_400);
    assert_eq!(options.query.end_unix_secs, 1_785_801_600);
    assert_eq!(options.query.step_secs, 3_600);
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(
        options.analytics.dashboard.source_endpoint,
        "https://example.com/api/v2"
    );
}

#[test]
fn analytics_total_supply_options_apply_bounded_defaults() {
    let (format, options) = parse_test_options(
        icrc_analytics_total_supply_command(),
        &[LEDGER_CANISTER_ID],
        |matches| analytics_total_supply_request(matches, 1_785_801_600),
    );

    assert_eq!(options.query.start_unix_secs, 1_785_801_600 - 30 * 86_400);
    assert_eq!(options.query.end_unix_secs, 1_785_801_600);
    assert_eq!(options.query.step_secs, 86_400);
    assert_eq!(format, OutputFormat::Text);
    assert_eq!(
        options.analytics.dashboard.source_endpoint,
        "https://icrc-api.internetcomputer.org/api/v2"
    );
}

#[test]
fn analytics_total_supply_step_values_keep_the_official_discrete_set() {
    for (step, expected) in [("3600", 3_600), ("86400", 86_400)] {
        let (_, options) = parse_test_options(
            icrc_analytics_total_supply_command(),
            &[LEDGER_CANISTER_ID, "--step", step],
            |matches| analytics_total_supply_request(matches, 1_785_801_600),
        );
        assert_eq!(options.query.step_secs, expected);
    }
    for step in ["7200", "0", "invalid"] {
        let error = icrc_analytics_total_supply_command()
            .try_get_matches_from(["total-supply", LEDGER_CANISTER_ID, "--step", step])
            .expect_err("unsupported step must fail during parsing");
        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
    }
}

#[test]
fn analytics_token_value_options_parse_bounds_limit_and_shared_target() {
    let (format, options) = parse_test_options(
        icrc_analytics_token_values_command(),
        &[
            LEDGER_CANISTER_ID,
            "--start",
            "1785542400",
            "--end",
            "1785628800",
            "--limit",
            "100",
            "--json",
            "--source-endpoint",
            "https://example.com/api/v2",
        ],
        |matches| analytics_token_value_request(matches, 1_785_801_600),
    );

    assert_eq!(options.analytics.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.query.start_unix_secs, 1_785_542_400);
    assert_eq!(options.query.end_unix_secs, 1_785_628_800);
    assert_eq!(options.query.limit, 100);
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(
        options.analytics.dashboard.source_endpoint,
        "https://example.com/api/v2"
    );

    let (_, defaults) = parse_test_options(
        icrc_analytics_token_values_command(),
        &[LEDGER_CANISTER_ID],
        |matches| analytics_token_value_request(matches, 1_785_801_600),
    );
    assert_eq!(defaults.query.start_unix_secs, 1_785_801_600 - 86_400);
    assert_eq!(defaults.query.end_unix_secs, 1_785_801_600);
    assert_eq!(defaults.query.limit, DEFAULT_ICRC_TOKEN_VALUE_LIMIT);
}

#[test]
fn analytics_indexed_count_options_reuse_the_shared_ledger_target() {
    for entity in ["account", "holder", "transaction"] {
        let options = parse_test_options(
            icrc_command_at(&["analytics", entity, "count"]),
            &[
                LEDGER_CANISTER_ID,
                "--json",
                "--source-endpoint",
                "https://example.com/api/v2",
            ],
            IcrcLedgerOptions::from_matches,
        );

        assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(options.format, OutputFormat::Json);
        assert_eq!(options.source_endpoint, "https://example.com/api/v2");
    }
}

#[test]
fn analytics_account_list_options_preserve_cursor_owner_and_descending_sort() {
    let (format, options) = parse_test_options(
        icrc_analytics_account_list_command(),
        &[
            LEDGER_CANISTER_ID,
            "--owner",
            ACCOUNT_OWNER,
            "--after",
            "1668734888.0,hkmli-faaaa-aaaar-qb4ba-cai",
            "--limit",
            "25",
            "--sort-by=-balance",
            "--json",
        ],
        |matches| analytics_account_list_request(matches, 1_785_801_600),
    );

    assert_eq!(options.query.owner.as_deref(), Some(ACCOUNT_OWNER));
    assert_eq!(
        options.query.after.as_deref(),
        Some("1668734888.0,hkmli-faaaa-aaaar-qb4ba-cai")
    );
    assert_eq!(options.query.before, None);
    assert_eq!(options.query.limit, 25);
    assert_eq!(options.query.sort_by, IcIcrcAccountSort::BalanceDescending);
    assert_eq!(format, OutputFormat::Json);
}

#[test]
fn analytics_sort_values_preserve_official_spelling_and_reject_unknown_values() {
    for value in [
        "id",
        "-id",
        "balance",
        "-balance",
        "total_transactions",
        "-total_transactions",
        "created_timestamp",
        "-created_timestamp",
        "owner",
        "-owner",
    ] {
        let (_, options) = parse_test_options(
            icrc_analytics_account_list_command(),
            &[LEDGER_CANISTER_ID, "--sort-by", value],
            |matches| analytics_account_list_request(matches, 1_785_801_600),
        );
        assert_eq!(options.query.sort_by.as_api_value(), value);
    }
    for value in [
        "balance",
        "-balance",
        "total_transactions",
        "-total_transactions",
        "created_timestamp",
        "-created_timestamp",
        "principal",
        "-principal",
    ] {
        let (_, options) = parse_test_options(
            icrc_analytics_holder_list_command(),
            &[LEDGER_CANISTER_ID, "--sort-by", value],
            |matches| analytics_holder_list_request(matches, 1_785_801_600),
        );
        assert_eq!(options.query.sort_by.as_api_value(), value);
    }
    for command in [
        icrc_analytics_account_list_command(),
        icrc_analytics_holder_list_command(),
    ] {
        let error = parse_matches(
            command,
            [LEDGER_CANISTER_ID, "--sort-by", "invalid"].map(std::ffi::OsString::from),
        )
        .expect_err("unknown sort value must fail in Clap");
        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
    }
}

#[test]
fn analytics_holder_list_and_account_info_options_use_operation_specific_contracts() {
    let (_, holder) = parse_test_options(
        icrc_analytics_holder_list_command(),
        &[LEDGER_CANISTER_ID, "--before", "holder-cursor"],
        |matches| analytics_holder_list_request(matches, 1_785_801_600),
    );
    assert_eq!(holder.query.after, None);
    assert_eq!(holder.query.before.as_deref(), Some("holder-cursor"));
    assert_eq!(holder.query.limit, 20);
    assert_eq!(holder.query.sort_by, IcIcrcHolderSort::Principal);

    let (_, info) = parse_test_options(
        icrc_analytics_account_info_command(),
        &[LEDGER_CANISTER_ID, ACCOUNT_OWNER],
        |matches| analytics_account_info_request(matches, 1_785_801_600),
    );
    assert_eq!(info.account_id, ACCOUNT_OWNER);
    assert_eq!(
        info.analytics.dashboard.source_endpoint,
        DEFAULT_ICRC_ACCOUNT_INFO_SOURCE_ENDPOINT
    );
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
        (root.as_str(), "analytics"),
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

#[test]
fn usage_mentions_icrc_analytics_surface() {
    let analytics = render_help(icrc_analytics_command());
    let analytics_token_values = render_help(icrc_analytics_token_values_command());
    let analytics_total_supply = render_help(icrc_analytics_total_supply_command());
    let analytics_account_info = render_help(icrc_analytics_account_info_command());
    let analytics_account_list = render_help(icrc_analytics_account_list_command());
    let analytics_holder_list = render_help(icrc_analytics_holder_list_command());
    let count_usages = ["account", "holder", "transaction"].map(|entity| {
        (
            entity,
            render_help(icrc_command_at(&["analytics", entity])),
            render_help(icrc_command_at(&["analytics", entity, "count"])),
        )
    });

    for needle in [
        "account",
        "holder",
        "token-values",
        "total-supply",
        "transaction",
    ] {
        assert!(analytics.contains(needle));
    }
    for (entity, namespace, count) in count_usages {
        assert!(namespace.contains("count"));
        assert!(count.contains("ledger-canister-id"));
        assert!(count.contains(&format!("current indexed {entity} count")));
        assert!(count.contains("Official IC Dashboard ICRC analytics API"));
        assert!(count.contains("Collection mode: Live query"));
    }
    for needle in ["ledger-canister-id", "--start", "--end", "--step"] {
        assert!(analytics_total_supply.contains(needle));
    }
    for needle in ["ledger-canister-id", "--start", "--end", "--limit"] {
        assert!(analytics_token_values.contains(needle));
    }
    assert!(analytics_token_values.contains("bounded external token value observations"));
    assert!(analytics_token_values.contains("Collection mode: Live query"));
    assert!(analytics_total_supply.contains("bounded historical total supply"));
    assert!(analytics_total_supply.contains("Official IC Dashboard ICRC analytics API"));
    assert!(analytics_total_supply.contains("Collection mode: Live query"));
    for usage in [
        analytics_account_info,
        analytics_account_list,
        analytics_holder_list,
    ] {
        assert!(usage.contains("Collection mode: Live query"));
        assert!(usage.contains("off-chain"));
        assert!(usage.contains("not point-in-time") || usage.contains("not a complete"));
    }
}
