use super::commands::test_support::{
    account_usage, allowance_usage, archives_usage, balance_usage, block_types_usage,
    capabilities_usage, index_usage, ledger_usage, parse_allowance_options, parse_archives_options,
    parse_balance_options, parse_block_types_options, parse_capabilities_options,
    parse_index_options, parse_tip_certificate_options, parse_token_options,
    parse_transactions_options, root_usage, tip_certificate_usage, token_usage, transactions_usage,
};
use crate::cli::common::OutputFormat;

const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const INDEX_CANISTER_ID: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const ACCOUNT_OWNER: &str = "aaaaa-aa";
const SOURCE_ENDPOINT: &str = "https://icp-api.io";
const SUBACCOUNT: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

#[test]
fn token_options_parse_through_clap() {
    let options = parse_token_options(&[
        LEDGER_CANISTER_ID,
        "--format",
        "json",
        "--source-endpoint",
        SOURCE_ENDPOINT,
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, SOURCE_ENDPOINT);
}

#[test]
fn capabilities_options_parse_through_clap() {
    let options = parse_capabilities_options(&[LEDGER_CANISTER_ID, "--format", "json"]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn balance_options_parse_through_clap_and_normalize_subaccount() {
    let options = parse_balance_options(&[
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        "--subaccount",
        SUBACCOUNT,
        "--format",
        "json",
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.subaccount_hex.as_deref(), Some(SUBACCOUNT));
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn allowance_options_parse_through_clap_and_normalize_subaccounts() {
    let options = parse_allowance_options(&[
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        ACCOUNT_OWNER,
        "--owner-subaccount",
        SUBACCOUNT,
        "--spender-subaccount",
        SUBACCOUNT,
    ]);

    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(options.spender_owner, ACCOUNT_OWNER);
    assert_eq!(options.account_subaccount_hex.as_deref(), Some(SUBACCOUNT));
    assert_eq!(options.spender_subaccount_hex.as_deref(), Some(SUBACCOUNT));
}

#[test]
fn index_options_parse_through_clap() {
    let options = parse_index_options(&[LEDGER_CANISTER_ID, "--format", "json"]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn transactions_options_parse_through_clap() {
    let options = parse_transactions_options(&[
        LEDGER_CANISTER_ID,
        "--start",
        "17",
        "--limit",
        "42",
        "--follow-archives",
    ]);

    assert_eq!(options.start, 17);
    assert_eq!(options.limit, 42);
    assert!(options.follow_archives);
}

#[test]
fn block_types_options_parse_through_clap() {
    let options = parse_block_types_options(&[LEDGER_CANISTER_ID, "--format", "json"]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn archives_options_parse_through_clap() {
    let options = parse_archives_options(&[
        LEDGER_CANISTER_ID,
        "--from",
        INDEX_CANISTER_ID,
        "--format",
        "json",
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.from_canister_id.as_deref(), Some(INDEX_CANISTER_ID));
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn tip_certificate_options_parse_through_clap() {
    let options = parse_tip_certificate_options(&[LEDGER_CANISTER_ID, "--format", "json"]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
}

#[test]
fn usage_mentions_icrc_command_surface() {
    for (usage, needle) in [
        (root_usage(), "ledger"),
        (root_usage(), "account"),
        (ledger_usage(), "capabilities"),
        (ledger_usage(), "transactions"),
        (account_usage(), "balance"),
        (account_usage(), "allowance"),
        (token_usage(), "ledger-canister-id"),
        (capabilities_usage(), "ledger-canister-id"),
        (balance_usage(), "principal"),
        (allowance_usage(), "spender-principal"),
        (index_usage(), "ledger-canister-id"),
        (transactions_usage(), "follow-archives"),
        (block_types_usage(), "ledger-canister-id"),
        (archives_usage(), "--from"),
        (tip_certificate_usage(), "ledger-canister-id"),
    ] {
        assert!(usage.contains(needle), "missing {needle:?} in {usage}");
    }

    assert!(token_usage().contains("icq icrc ledger token"));
    assert!(balance_usage().contains("icq icrc account balance"));

    for usage in [
        token_usage(),
        capabilities_usage(),
        balance_usage(),
        allowance_usage(),
        index_usage(),
        transactions_usage(),
        block_types_usage(),
        archives_usage(),
        tip_certificate_usage(),
    ] {
        assert!(usage.contains("Collection mode: Live query"));
    }
}
