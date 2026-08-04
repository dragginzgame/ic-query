use std::{
    fs,
    path::Path,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn run_icq(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_icq"))
        .args(args)
        .output()
        .expect("run icq test binary")
}

fn run_icq_in_root(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_icq"))
        .env("ICQ_CACHE_ROOT", root)
        .args(args)
        .output()
        .expect("run icq test binary")
}

fn run_icq_with_xdg_cache(cwd: &Path, xdg_cache_home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_icq"))
        .current_dir(cwd)
        .env_remove("ICQ_CACHE_ROOT")
        .env("XDG_CACHE_HOME", xdg_cache_home)
        .args(args)
        .output()
        .expect("run icq test binary")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("icq stdout is utf-8")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("icq stderr is utf-8")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success, got status {:?}\nstderr:\n{}",
        output.status.code(),
        stderr_text(output)
    );
}

fn temp_cache_root(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()))
}

#[test]
fn binary_top_level_help_smoke() {
    let output = run_icq(&["help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq [OPTIONS] [COMMAND]"));
    assert!(stdout.contains("ic"));
    assert!(stdout.contains("icrc"));
    assert!(stdout.contains("nns"));
    assert!(stdout.contains("sns"));
    assert!(stdout.contains("system"));
}

#[test]
fn binary_system_report_help_smoke() {
    for (command, description) in [
        ("xdr", "certified CMC ICP/XDR conversion rate"),
        (
            "cycles",
            "cycles conversions derived from the certified CMC rate",
        ),
    ] {
        let output = run_icq(&["system", command, "--help"]);

        assert_success(&output);
        let stdout = stdout_text(&output);
        assert!(stdout.contains(&format!("Usage: icq system {command} [OPTIONS]")));
        assert!(stdout.contains("--source-endpoint <url>"));
        assert!(stdout.contains("--json"));
        assert!(stdout.contains(description));
        assert!(stdout.contains("Live query; does not read or write a report cache."));
    }
}

#[test]
fn binary_top_level_help_after_global_options_succeeds() {
    let output = run_icq(&["--network", "ic", "--help"]);

    assert_success(&output);
    assert!(stdout_text(&output).contains("Usage: icq [OPTIONS] [COMMAND]"));
}

#[test]
fn binary_invalid_value_preserves_clap_diagnostic() {
    let output = run_icq(&["nns", "proposal", "list", "--limit", "nope"]);

    assert!(!output.status.success());
    let stderr = stderr_text(&output);
    assert!(stderr.contains("invalid value 'nope'"));
    assert!(stderr.contains("--limit <count>"));
}

#[test]
fn binary_invalid_network_precedes_help_like_option_values() {
    let output = run_icq(&[
        "--network",
        "local",
        "nns",
        "governance",
        "economics",
        "--source-endpoint",
        "help",
    ]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = stderr_text(&output);
    assert!(stderr.contains("invalid value 'local'"));
    assert!(stderr.contains("possible values: ic"));
    assert!(!stderr.contains("failed to build IC agent"));
}

#[test]
fn binary_ic_canister_info_help_smoke() {
    let output = run_icq(&["ic", "canister", "info", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq ic canister info [OPTIONS] <canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("off-chain analytics authority"));
}

#[test]
fn binary_ic_metrics_help_smoke() {
    let output = run_icq(&["ic", "metrics", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq ic metrics [OPTIONS] <metric>"));
    assert!(stdout.contains("instruction-rate"));
    assert!(stdout.contains("--start <unix-seconds>"));
    assert!(stdout.contains("--step <seconds>"));
    assert!(stdout.contains("one official Dashboard Metrics API request"));
}

#[test]
fn binary_ic_boundary_node_data_centers_help_smoke() {
    let output = run_icq(&["ic", "network", "boundary-node-data-centers", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq ic network boundary-node-data-centers [OPTIONS]"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("one official Dashboard v4 request"));
}

#[test]
fn binary_ic_daily_stats_help_smoke() {
    let output = run_icq(&["ic", "network", "daily-stats", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq ic network daily-stats [OPTIONS]"));
    assert!(stdout.contains("--start <unix-seconds>"));
    assert!(stdout.contains("--end <unix-seconds>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("one official Dashboard v3 request"));
}

#[test]
fn binary_icrc_balance_help_smoke() {
    let output = run_icq(&["icrc", "account", "balance", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout
            .contains("Usage: icq icrc account balance [OPTIONS] <ledger-canister-id> <principal>")
    );
    assert!(stdout.contains("--subaccount <hex>"));
    assert!(stdout.contains("--source-endpoint <url>"));
}

#[test]
fn binary_icrc_capabilities_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "capabilities", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger capabilities [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn binary_icrc_analytics_total_supply_help_smoke() {
    let output = run_icq(&["icrc", "analytics", "total-supply", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("Usage: icq icrc analytics total-supply [OPTIONS] <ledger-canister-id>")
    );
    assert!(stdout.contains("--start <unix-seconds>"));
    assert!(stdout.contains("--end <unix-seconds>"));
    assert!(stdout.contains("--step <seconds>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("exactly one live request"));
    assert!(stdout.contains("does not use a cache"));
}

#[test]
fn binary_icrc_analytics_token_values_help_smoke() {
    let output = run_icq(&["icrc", "analytics", "token-values", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("Usage: icq icrc analytics token-values [OPTIONS] <ledger-canister-id>")
    );
    assert!(stdout.contains("--start <unix-seconds>"));
    assert!(stdout.contains("--end <unix-seconds>"));
    assert!(stdout.contains("--limit <rows>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("exactly one live request"));
    assert!(stdout.contains("external provider name and URL"));
    assert!(stdout.contains("does not use a cache"));
}

#[test]
fn binary_icrc_analytics_indexed_count_help_smoke() {
    for (entity, plural) in [
        ("account", "accounts"),
        ("holder", "holders"),
        ("transaction", "transactions"),
    ] {
        let output = run_icq(&["icrc", "analytics", entity, "count", "--help"]);

        assert_success(&output);
        let stdout = stdout_text(&output);
        assert!(stdout.contains(&format!(
            "Usage: icq icrc analytics {entity} count [OPTIONS] <ledger-canister-id>"
        )));
        assert!(stdout.contains("--source-endpoint <url>"));
        assert!(stdout.contains("--json"));
        assert!(stdout.contains("exactly one live request"));
        assert!(stdout.contains(&format!("requests no {plural} rows")));
        assert!(stdout.contains("does not use a cache"));
    }
}

#[test]
fn binary_icrc_allowance_help_smoke() {
    let output = run_icq(&["icrc", "account", "allowance", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains(
        "Usage: icq icrc account allowance [OPTIONS] <ledger-canister-id> <owner-principal> <spender-principal>"
    ));
    assert!(stdout.contains("--owner-subaccount <hex>"));
    assert!(stdout.contains("--spender-subaccount <hex>"));
    assert!(stdout.contains("--source-endpoint <url>"));
}

#[test]
fn binary_icrc_account_transaction_help_smoke() {
    for (args, usage, option) in [
        (
            &["icrc", "account", "transaction", "page", "--help"][..],
            "Usage: icq icrc account transaction page [OPTIONS] <ledger-canister-id> <principal>",
            "--start <block-index>",
        ),
        (
            &["icrc", "account", "transaction", "list", "--help"][..],
            "Usage: icq icrc account transaction list [OPTIONS] <ledger-canister-id> <principal>",
            "--sort <newest|oldest>",
        ),
        (
            &["icrc", "account", "transaction", "refresh", "--help"][..],
            "Usage: icq icrc account transaction refresh [OPTIONS] <ledger-canister-id> <principal>",
            "--page-size <count>",
        ),
        (
            &[
                "icrc",
                "account",
                "transaction",
                "cache",
                "status",
                "--help",
            ][..],
            "Usage: icq icrc account transaction cache status [OPTIONS] <ledger-canister-id> <principal>",
            "--source-endpoint <url>",
        ),
    ] {
        let output = run_icq(args);
        assert_success(&output);
        let stdout = stdout_text(&output);
        assert!(stdout.contains(usage), "missing {usage:?} in {stdout}");
        assert!(stdout.contains(option), "missing {option:?} in {stdout}");
        assert!(stdout.contains("--json"));
    }
}

#[test]
fn binary_icrc_account_transaction_cache_status_is_local_only() {
    let root = temp_cache_root("ic-query-cli-icrc-account-status");

    let output = run_icq_in_root(
        &root,
        &[
            "icrc",
            "account",
            "transaction",
            "cache",
            "status",
            "ryjl3-tyaaa-aaaaa-aaaba-cai",
            "aaaaa-aa",
            "--json",
        ],
    );

    assert_success(&output);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse cache status JSON");
    assert_eq!(report["found"], false);
    assert!(output.stderr.is_empty());
    assert!(!root.exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn binary_default_cache_root_uses_xdg_cache_home() {
    let cwd = temp_cache_root("ic-query-cli-cwd");
    let xdg_cache_home = temp_cache_root("ic-query-cli-xdg");
    fs::create_dir_all(&cwd).expect("create temporary working directory");

    let output = run_icq_with_xdg_cache(
        &cwd,
        &xdg_cache_home,
        &["nns", "proposal", "cache", "status", "--json"],
    );

    assert_success(&output);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse cache status JSON");
    assert_eq!(
        report["cache_root"],
        xdg_cache_home
            .join("ic-query")
            .join("nns")
            .join("ic")
            .join("governance")
            .join("proposals")
            .display()
            .to_string()
    );
    assert!(!xdg_cache_home.exists());

    let _ = fs::remove_dir_all(cwd);
}

#[test]
fn binary_icrc_index_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "index", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger index [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn binary_icrc_transactions_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "transactions", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger transactions [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--start <index>"));
    assert!(stdout.contains("--limit <count>"));
    assert!(stdout.contains("--follow-archives"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn binary_icrc_block_types_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "block-types", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger block-types [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn binary_icrc_archives_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "archives", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger archives [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--from <canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn binary_icrc_tip_certificate_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "tip-certificate", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("Usage: icq icrc ledger tip-certificate [OPTIONS] <ledger-canister-id>")
    );
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn binary_sns_list_help_smoke() {
    let output = run_icq(&["sns", "list", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns list [OPTIONS]"));
    assert!(stdout.contains("--sort <id|name>"));
    assert!(stdout.contains("--verbose"));
}

#[test]
fn binary_command_namespaces_match_explicit_local_help() {
    let cases: &[(&[&str], &[&str])] = &[
        (&[], &["help"]),
        (&["ic"], &["ic", "help"]),
        (&["ic", "canister"], &["ic", "canister", "help"]),
        (&["ic", "network"], &["ic", "network", "help"]),
        (&["icrc"], &["icrc", "help"]),
        (&["icrc", "account"], &["icrc", "account", "help"]),
        (&["icrc", "analytics"], &["icrc", "analytics", "help"]),
        (
            &["icrc", "analytics", "account"],
            &["icrc", "analytics", "account", "help"],
        ),
        (
            &["icrc", "analytics", "holder"],
            &["icrc", "analytics", "holder", "help"],
        ),
        (
            &["icrc", "analytics", "transaction"],
            &["icrc", "analytics", "transaction", "help"],
        ),
        (
            &["icrc", "account", "transaction"],
            &["icrc", "account", "transaction", "help"],
        ),
        (
            &["icrc", "account", "transaction", "cache"],
            &["icrc", "account", "transaction", "cache", "help"],
        ),
        (&["icrc", "ledger"], &["icrc", "ledger", "help"]),
        (&["nns"], &["nns", "help"]),
        (&["nns", "data-center"], &["nns", "data-center", "help"]),
        (&["nns", "governance"], &["nns", "governance", "help"]),
        (&["nns", "neuron"], &["nns", "neuron", "help"]),
        (
            &["nns", "neuron", "cache"],
            &["nns", "neuron", "cache", "help"],
        ),
        (&["nns", "node"], &["nns", "node", "help"]),
        (&["nns", "node-operator"], &["nns", "node-operator", "help"]),
        (&["nns", "node-provider"], &["nns", "node-provider", "help"]),
        (&["nns", "proposal"], &["nns", "proposal", "help"]),
        (
            &["nns", "proposal", "cache"],
            &["nns", "proposal", "cache", "help"],
        ),
        (&["nns", "registry"], &["nns", "registry", "help"]),
        (&["nns", "subnet"], &["nns", "subnet", "help"]),
        (&["nns", "topology"], &["nns", "topology", "help"]),
        (&["sns"], &["sns", "help"]),
        (&["sns", "canister"], &["sns", "canister", "help"]),
        (&["sns", "neuron"], &["sns", "neuron", "help"]),
        (
            &["sns", "neuron", "cache"],
            &["sns", "neuron", "cache", "help"],
        ),
        (&["sns", "proposal"], &["sns", "proposal", "help"]),
        (
            &["sns", "proposal", "cache"],
            &["sns", "proposal", "cache", "help"],
        ),
        (&["sns", "reward"], &["sns", "reward", "help"]),
        (&["system"], &["system", "help"]),
    ];

    for (implicit_args, explicit_args) in cases {
        let implicit = run_icq(implicit_args);
        let explicit = run_icq(explicit_args);

        assert_success(&implicit);
        assert_success(&explicit);
        assert_eq!(stdout_text(&implicit), stdout_text(&explicit));
        assert!(stderr_text(&implicit).is_empty());
    }
}

#[test]
fn binary_sns_swap_help_smoke() {
    let output = run_icq(&["sns", "swap", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns swap [OPTIONS] <id|root-principal>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("exactly three bounded"));
}

#[test]
fn binary_sns_upgrade_help_smoke() {
    let output = run_icq(&["sns", "upgrade", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns upgrade [OPTIONS] <id|root-principal>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("at most four live calls"));
}

#[test]
fn binary_sns_metrics_help_smoke() {
    let output = run_icq(&["sns", "metrics", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns metrics [OPTIONS] <id|root-principal>"));
    assert!(stdout.contains("--window <duration>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("get_metrics composite query"));
}

#[test]
fn binary_sns_parameters_help_smoke() {
    let output = run_icq(&["sns", "parameters", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns parameters [OPTIONS] <id|root-principal>"));
    assert!(stdout.contains("nervous system parameters"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn binary_sns_canister_list_help_smoke() {
    let output = run_icq(&["sns", "canister", "list", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns canister list [OPTIONS] <id|root-principal>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("update_canister_list=false"));
}

#[test]
fn binary_sns_nested_family_help_uses_operation_before_target() {
    for (args, usage) in [
        (
            &["sns", "neuron", "list", "--help"][..],
            "Usage: icq sns neuron list [OPTIONS] <id|root-principal>",
        ),
        (
            &["sns", "proposal", "info", "--help"][..],
            "Usage: icq sns proposal info [OPTIONS] <id|root-principal> <proposal-id>",
        ),
    ] {
        let output = run_icq(args);
        assert_success(&output);
        assert!(stdout_text(&output).contains(usage));
    }
}

#[test]
fn binary_sns_reward_checkpoint_help_smoke() {
    let output = run_icq(&["sns", "reward", "checkpoint", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns reward checkpoint [OPTIONS] <id|root-principal>"));
    assert!(stdout.contains("--max-pages <count>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("N + 8 client queries"));
}

#[test]
fn binary_sns_reward_diff_help_is_local_only() {
    let output = run_icq(&["sns", "reward", "diff", "--help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns reward diff [OPTIONS] <before.json> <after.json>"));
    assert!(stdout.contains("Local-only file inspection"));
    assert!(stdout.contains("--json"));
    assert!(!stdout.contains("--source-endpoint"));
}

#[test]
fn binary_nns_topology_help_smoke() {
    let output = run_icq(&["help", "nns", "topology"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq nns topology [COMMAND]"));
    assert!(stdout.contains("summary"));
    assert!(stdout.contains("refresh"));
}

#[test]
fn binary_version_smoke() {
    let output = run_icq(&["--version"]);

    assert_success(&output);
    assert_eq!(
        stdout_text(&output),
        format!("icq {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn binary_local_cache_commands_emit_json_without_live_calls() {
    let root = temp_cache_root("ic-query-cli-cache-json");
    fs::create_dir_all(&root).expect("create temporary cache root");

    let nns_status = run_icq_in_root(&root, &["nns", "proposal", "cache", "status", "--json"]);
    assert_success(&nns_status);
    let nns_status: serde_json::Value =
        serde_json::from_str(&stdout_text(&nns_status)).expect("nns cache status json");
    assert_eq!(nns_status["found"], false);

    let sns_proposals = run_icq_in_root(&root, &["sns", "proposal", "cache", "list", "--json"]);
    assert_success(&sns_proposals);
    let sns_proposals: serde_json::Value =
        serde_json::from_str(&stdout_text(&sns_proposals)).expect("sns proposals cache list json");
    assert_eq!(sns_proposals["cache_count"], 0);

    let sns_neurons = run_icq_in_root(&root, &["sns", "neuron", "cache", "list", "--json"]);
    assert_success(&sns_neurons);
    let sns_neurons: serde_json::Value =
        serde_json::from_str(&stdout_text(&sns_neurons)).expect("sns neurons cache list json");
    assert_eq!(sns_neurons["cache_count"], 0);

    let _ = fs::remove_dir_all(root);
}
