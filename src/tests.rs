use super::*;
use crate::cli::globals::INTERNAL_NETWORK_OPTION;
use crate::test_support::assert_snapshot;
use crate::{
    duration::{display_duration_seconds, parse_duration_seconds},
    table::{ColumnAlign, render_table},
    token_amount::{base_units_decimal_text, e8s_decimal_text},
};
use std::ffi::OsString;

#[test]
fn usage_lists_query_families() {
    let text = usage();

    assert!(text.contains("Usage: icq [OPTIONS] [COMMAND]"));
    assert!(text.contains("nns"));
    assert!(text.contains("Inspect NNS metadata"));
    assert!(text.contains("sns"));
    assert!(text.contains("Inspect SNS metadata"));
    assert!(text.contains("Run `icq <command> help`"));
}

#[test]
fn top_level_usage_snapshot() {
    let expected = format!(
        "\
icq {}
Internet Computer metadata query CLI

Usage: icq [OPTIONS] [COMMAND]

Commands:
  nns  Inspect NNS metadata
  sns  Inspect SNS metadata

Options:
  -V, --version         Print version
      --network <name>  ICP CLI network for networked commands
  -h, --help            Print help

Run `icq <command> help` for command-specific help.
",
        env!("CARGO_PKG_VERSION")
    );

    assert_snapshot("top-level usage", &usage(), &expected);
}

#[test]
fn command_family_help_returns_ok() {
    for args in [
        &["nns", "help"][..],
        &["nns", "data-center", "help"],
        &["nns", "data-center", "list", "help"],
        &["nns", "data-center", "info", "help"],
        &["nns", "data-center", "refresh", "help"],
        &["nns", "node", "help"],
        &["nns", "node", "list", "help"],
        &["nns", "node", "info", "help"],
        &["nns", "node", "refresh", "help"],
        &["nns", "node-provider", "help"],
        &["nns", "node-provider", "list", "help"],
        &["nns", "node-provider", "info", "help"],
        &["nns", "node-provider", "refresh", "help"],
        &["nns", "node-operator", "help"],
        &["nns", "node-operator", "list", "help"],
        &["nns", "node-operator", "info", "help"],
        &["nns", "node-operator", "refresh", "help"],
        &["nns", "proposal", "help"],
        &["nns", "proposal", "list", "help"],
        &["nns", "proposal", "info", "help"],
        &["nns", "registry", "help"],
        &["nns", "registry", "version", "help"],
        &["nns", "subnet", "help"],
        &["nns", "subnet", "list", "help"],
        &["nns", "subnet", "info", "help"],
        &["nns", "subnet", "refresh", "help"],
        &["nns", "topology", "help"],
        &["nns", "topology", "summary", "help"],
        &["nns", "topology", "coverage", "help"],
        &["nns", "topology", "versions", "help"],
        &["nns", "topology", "health", "help"],
        &["nns", "topology", "gaps", "help"],
        &["nns", "topology", "capacity", "help"],
        &["nns", "topology", "regions", "help"],
        &["nns", "topology", "providers", "help"],
        &["nns", "topology", "refresh", "help"],
        &["sns", "help"],
        &["sns", "list", "help"],
        &["sns", "info", "help"],
        &["sns", "token", "help"],
        &["sns", "params", "help"],
        &["sns", "proposal", "help"],
        &["sns", "proposals", "help"],
        &["sns", "neurons", "help"],
        &["sns", "neurons", "cache", "help"],
        &["sns", "neurons", "cache", "list", "help"],
        &["sns", "neurons", "cache", "status", "help"],
        &["sns", "neurons", "refresh", "help"],
    ] {
        assert_run_ok(args);
    }
}

#[test]
fn version_flags_return_ok() {
    assert_eq!(version_text(), concat!("icq ", env!("CARGO_PKG_VERSION")));
    assert!(run([OsString::from("--version")]).is_ok());
    assert!(run([OsString::from("nns"), OsString::from("--version")]).is_ok());
    assert!(run([OsString::from("sns"), OsString::from("--version")]).is_ok());
    assert!(
        run([
            OsString::from("nns"),
            OsString::from("subnet"),
            OsString::from("list"),
            OsString::from("--version")
        ])
        .is_ok()
    );

    let mut sns_info_tail = vec![OsString::from("info"), OsString::from("1")];

    cli::globals::apply_global_network("sns", &mut sns_info_tail, Some("ic".to_string()));

    assert_eq!(
        sns_info_tail,
        vec![
            OsString::from("info"),
            OsString::from("1"),
            OsString::from(INTERNAL_NETWORK_OPTION),
            OsString::from("ic")
        ]
    );
}

#[test]
fn global_network_is_forwarded_to_networked_leaf_commands() {
    let mut nns_tail = vec![OsString::from("data-center"), OsString::from("list")];

    cli::globals::apply_global_network("nns", &mut nns_tail, Some("ic".to_string()));

    assert_eq!(
        nns_tail,
        vec![
            OsString::from("data-center"),
            OsString::from("list"),
            OsString::from(INTERNAL_NETWORK_OPTION),
            OsString::from("ic")
        ]
    );

    let mut sns_tail = vec![OsString::from("list")];

    cli::globals::apply_global_network("sns", &mut sns_tail, Some("ic".to_string()));

    assert_eq!(
        sns_tail,
        vec![
            OsString::from("list"),
            OsString::from(INTERNAL_NETWORK_OPTION),
            OsString::from("ic")
        ]
    );
}

#[test]
fn sns_nested_commands_dispatch_through_clap_subcommands() {
    assert!(
        run([
            OsString::from("sns"),
            OsString::from("neurons"),
            OsString::from("refresh"),
            OsString::from("--help")
        ])
        .is_ok()
    );
    assert!(
        run([
            OsString::from("sns"),
            OsString::from("proposals"),
            OsString::from("cache"),
            OsString::from("status"),
            OsString::from("--help")
        ])
        .is_ok()
    );
}

#[test]
fn base_units_render_as_two_decimal_token_amounts() {
    assert_eq!(base_units_decimal_text("0", 8), "0.00");
    assert_eq!(base_units_decimal_text("000000000", 8), "0.00");
    assert_eq!(base_units_decimal_text("10_000", 8), "0.00");
    assert_eq!(
        base_units_decimal_text("100_923_109_141_460", 8),
        "1009231.09"
    );
    assert_eq!(base_units_decimal_text("500000", 8), "0.01");
    assert_eq!(base_units_decimal_text("123456789", 8), "1.23");
    assert_eq!(base_units_decimal_text("123500000", 8), "1.24");
    assert_eq!(base_units_decimal_text("3000000000000", 8), "30000.00");
    assert_eq!(base_units_decimal_text("123", 0), "123.00");
    assert_eq!(base_units_decimal_text("123", 1), "12.30");
    assert_eq!(base_units_decimal_text("123", 2), "1.23");
    assert_eq!(base_units_decimal_text("999", 3), "1.00");
    assert_eq!(base_units_decimal_text("not-a-number", 8), "not-a-number");
}

#[test]
fn e8s_render_as_two_decimal_token_amounts() {
    assert_eq!(e8s_decimal_text(0), "0.00");
    assert_eq!(e8s_decimal_text(123), "0.00");
    assert_eq!(e8s_decimal_text(499_999), "0.00");
    assert_eq!(e8s_decimal_text(500_000), "0.01");
    assert_eq!(e8s_decimal_text(100_000_000), "1.00");
    assert_eq!(e8s_decimal_text(123_456_789), "1.23");
    assert_eq!(e8s_decimal_text(123_500_000), "1.24");
    assert_eq!(e8s_decimal_text(3_000_000_000_000), "30000.00");
}

#[test]
fn render_table_handles_long_left_aligned_cells() {
    let rows = [[
        "ICRC-1".to_string(),
        "https://github.com/dfinity/ICRC-1?with=a-long-token-metadata-url".to_string(),
    ]];

    let table = render_table(
        &["STANDARD", "URL"],
        &rows,
        &[ColumnAlign::Left, ColumnAlign::Left],
    );

    assert!(table.contains("ICRC-1"));
    assert!(table.contains("a-long-token-metadata-url"));
}

#[test]
fn render_table_right_aligns_cells() {
    let rows = [["1".to_string(), "Dragginz".to_string()]];

    let table = render_table(
        &["ID", "NAME"],
        &rows,
        &[ColumnAlign::Right, ColumnAlign::Left],
    );

    assert!(table.contains("ID   NAME"));
    assert!(table.contains(" 1   Dragginz"));
}

#[test]
fn duration_display_uses_largest_readable_unit() {
    assert_eq!(display_duration_seconds(0), "0s");
    assert_eq!(display_duration_seconds(86_400), "1d");
    assert_eq!(display_duration_seconds(2_629_800), "30.44d");
    assert_eq!(display_duration_seconds(5_400), "1.50h");
    assert_eq!(display_duration_seconds(90), "1.50m");
    assert_eq!(display_duration_seconds(45), "45s");
}

#[test]
fn duration_parser_accepts_integer_units() {
    assert_eq!(parse_duration_seconds("45").expect("seconds"), 45);
    assert_eq!(parse_duration_seconds("30m").expect("minutes"), 1_800);
    assert_eq!(parse_duration_seconds("2h").expect("hours"), 7_200);
    assert_eq!(parse_duration_seconds("1d").expect("days"), 86_400);
    assert!(parse_duration_seconds("0").is_err());
    assert!(parse_duration_seconds("1.5h").is_err());
}

fn assert_run_ok(args: &[&str]) {
    let args = args.iter().copied().map(OsString::from).collect::<Vec<_>>();
    if let Err(err) = run(args.clone()) {
        panic!("expected {args:?} to succeed, got {err}");
    }
}
