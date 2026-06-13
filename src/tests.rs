use super::*;
use crate::cli::globals::INTERNAL_NETWORK_OPTION;
use crate::test_support::assert_snapshot;
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
        &["sns", "neurons", "help"],
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

fn assert_run_ok(args: &[&str]) {
    let args = args.iter().copied().map(OsString::from).collect::<Vec<_>>();
    if let Err(err) = run(args.clone()) {
        panic!("expected {args:?} to succeed, got {err}");
    }
}
