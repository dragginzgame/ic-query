use super::*;

#[test]
fn usage_lists_query_families_and_native_help_guidance() {
    let text = usage();

    assert!(text.contains("Usage: icq [OPTIONS] [COMMAND]"));
    assert!(text.contains("ic"));
    assert!(text.contains("Inspect certified IC state and official Dashboard data"));
    assert!(text.contains("cache"));
    assert!(text.contains("Inspect the local ic-query cache"));
    assert!(text.contains("cloud-engine"));
    assert!(text.contains("Inspect public CloudEngine metadata"));
    assert!(text.contains("icrc"));
    assert!(text.contains("Inspect generic ICRC ledgers"));
    assert!(text.contains("nns"));
    assert!(text.contains("Inspect NNS metadata"));
    assert!(text.contains("sns"));
    assert!(text.contains("Inspect SNS metadata"));
    assert!(text.contains("system"));
    assert!(text.contains("Inspect native IC system-canister metadata"));
    assert!(text.contains("Run `icq <command> --help`"));
}

#[test]
fn every_subcommand_uses_alphabetical_help_order() {
    fn assert_equal_display_order(command: &Command, path: &mut Vec<String>) {
        for subcommand in command.get_subcommands() {
            path.push(subcommand.get_name().to_string());
            assert_eq!(
                subcommand.get_display_order(),
                0,
                "non-alphabetical display rank for {}",
                path.join(" ")
            );
            assert_equal_display_order(subcommand, path);
            path.pop();
        }
    }

    assert_equal_display_order(&cli_command(), &mut vec!["icq".to_string()]);
}

#[test]
fn every_command_namespace_defaults_to_local_help() {
    fn assert_namespace_policy(command: &Command, path: &mut Vec<String>) {
        let has_operational_subcommands = command
            .get_subcommands()
            .any(|subcommand| subcommand.get_name() != "help");
        if has_operational_subcommands {
            assert!(
                command.is_arg_required_else_help_set(),
                "missing default help policy for {}",
                path.join(" ")
            );
            assert!(
                !command.is_subcommand_required_set(),
                "terse missing-subcommand policy remains on {}",
                path.join(" ")
            );
        }

        for subcommand in command
            .get_subcommands()
            .filter(|subcommand| subcommand.get_name() != "help")
        {
            path.push(subcommand.get_name().to_string());
            assert_namespace_policy(subcommand, path);
            path.pop();
        }
    }

    assert_namespace_policy(&cli_command(), &mut vec!["icq".to_string()]);
}

#[tokio::test(flavor = "current_thread")]
async fn native_help_and_propagated_version_return_without_dispatch() {
    for args in [
        &["--help"][..],
        &["ic", "canister", "info", "--help"],
        &["cache", "status", "--help"],
        &[
            "icrc",
            "account",
            "transaction",
            "cache",
            "status",
            "--help",
        ],
        &["cloud-engine", "info", "--help"],
        &["cloud-engine", "node", "list", "--help"],
        &["cloud-engine", "provider", "list", "--help"],
        &["nns", "topology", "providers", "--help"],
        &["sns", "proposal", "cache", "status", "--help"],
        &["system", "cycles", "--help"],
        &["--version"],
        &["nns", "subnet", "list", "--version"],
    ] {
        assert_run_ok(args).await;
    }
}

#[test]
fn every_composed_command_path_supports_native_help() {
    fn collect_paths(
        command: &Command,
        prefix: &mut Vec<OsString>,
        paths: &mut Vec<Vec<OsString>>,
    ) {
        for subcommand in command.get_subcommands() {
            prefix.push(OsString::from(subcommand.get_name()));
            paths.push(prefix.clone());
            collect_paths(subcommand, prefix, paths);
            prefix.pop();
        }
    }

    let mut paths = Vec::new();
    collect_paths(&top_level_command(), &mut Vec::new(), &mut paths);
    assert_ne!(paths, Vec::<Vec<OsString>>::new());

    for mut path in paths {
        path.push(OsString::from("--help"));
        let error = parse_matches(top_level_command(), path.clone())
            .expect_err("native help must stop before typed dispatch");
        assert_eq!(
            error.kind(),
            ErrorKind::DisplayHelp,
            "unexpected result for {path:?}"
        );
    }
}

#[test]
fn every_report_leaf_exposes_the_shared_json_flag() {
    fn assert_leaf_json(command: &Command, path: &mut Vec<String>) {
        let subcommands = command.get_subcommands().collect::<Vec<_>>();
        if subcommands.is_empty() {
            assert!(
                command
                    .get_arguments()
                    .any(|argument| argument.get_id() == "json"),
                "missing --json on {}",
                path.join(" ")
            );
            return;
        }

        for subcommand in subcommands {
            path.push(subcommand.get_name().to_string());
            assert_leaf_json(subcommand, path);
            path.pop();
        }
    }

    assert_leaf_json(&top_level_command(), &mut vec!["icq".to_string()]);
}

#[tokio::test(flavor = "current_thread")]
async fn clap_rejects_non_mainnet_and_command_local_network_options() {
    let error = run([
        OsString::from("--network"),
        OsString::from("local"),
        OsString::from("nns"),
        OsString::from("registry"),
        OsString::from("version"),
    ])
    .await
    .expect_err("non-mainnet network must fail in Clap");
    assert_eq!(error.exit_code(), 2);
    assert!(error.to_string().contains("invalid value 'local'"));

    let error = run([
        OsString::from("nns"),
        OsString::from("registry"),
        OsString::from("version"),
        OsString::from("--network"),
        OsString::from("ic"),
    ])
    .await
    .expect_err("network remains a top-level option");
    assert_eq!(error.exit_code(), 2);
    assert!(
        error
            .to_string()
            .contains("unexpected argument '--network'")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn network_is_rejected_for_endpoint_identified_families() {
    for args in [
        &["--network", "ic", "ic", "canister", "count"][..],
        &[
            "--network",
            "ic",
            "icrc",
            "ledger",
            "token",
            "ryjl3-tyaaa-aaaaa-aaaba-cai",
        ],
    ] {
        let error = run(args.iter().map(OsString::from))
            .await
            .expect_err("endpoint-identified families must reject --network");
        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("--source-endpoint"));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn explicit_network_is_rejected_for_cross_network_cache_status() {
    let error = run([
        OsString::from("--network"),
        OsString::from("ic"),
        OsString::from("cache"),
        OsString::from("status"),
    ])
    .await
    .expect_err("cross-network cache status must reject one selected network");

    assert_eq!(error.exit_code(), 2);
    assert!(error.to_string().contains("every network"));
}

#[tokio::test(flavor = "current_thread")]
async fn targeted_sns_leaves_require_their_identifiers() {
    for args in [
        &["sns", "neuron", "list"][..],
        &["sns", "proposal", "refresh"][..],
    ] {
        let error = run(args.iter().map(OsString::from))
            .await
            .expect_err("targeted SNS operation must require an SNS selector");
        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("<id|root-principal>"));
    }
}

#[test]
fn typed_cli_errors_preserve_exit_and_broken_pipe_semantics() {
    for usage in [
        IcqCliError::Ic(ic::IcCommandError::Usage("bad input".to_string())),
        IcqCliError::Icrc(icrc::IcrcCommandError::Usage("bad input".to_string())),
        IcqCliError::System(system::SystemCommandError::Usage("bad input".to_string())),
    ] {
        assert_eq!(usage.exit_code(), 2);
        assert!(!usage.is_broken_pipe());
    }

    for broken_pipe in [
        IcqCliError::CloudEngine(cloud_engine::CloudEngineCommandError::Io(
            std::io::Error::from(std::io::ErrorKind::BrokenPipe),
        )),
        IcqCliError::Ic(ic::IcCommandError::Io(std::io::Error::from(
            std::io::ErrorKind::BrokenPipe,
        ))),
        IcqCliError::Icrc(icrc::IcrcCommandError::Io(std::io::Error::from(
            std::io::ErrorKind::BrokenPipe,
        ))),
        IcqCliError::System(system::SystemCommandError::Io(std::io::Error::from(
            std::io::ErrorKind::BrokenPipe,
        ))),
    ] {
        assert_eq!(broken_pipe.exit_code(), 1);
        assert!(broken_pipe.is_broken_pipe());
    }
}

async fn assert_run_ok(args: &[&str]) {
    let args = args.iter().copied().map(OsString::from).collect::<Vec<_>>();
    if let Err(err) = run(args.clone()).await {
        panic!("expected {args:?} to succeed, got {err}");
    }
}
