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
        let output = run_icq(&["system", command, "help"]);

        assert_success(&output);
        let stdout = stdout_text(&output);
        assert!(stdout.contains(&format!("Usage: icq system {command} [OPTIONS]")));
        assert!(stdout.contains("--source-endpoint <url>"));
        assert!(stdout.contains("--format <text|json>"));
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
fn binary_ic_canister_info_help_smoke() {
    let output = run_icq(&["ic", "canister", "info", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq ic canister info [OPTIONS] <canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--format <text|json>"));
    assert!(stdout.contains("off-chain analytics authority"));
}

#[test]
fn binary_ic_metrics_help_smoke() {
    let output = run_icq(&["ic", "metrics", "help"]);

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
    let output = run_icq(&["ic", "network", "boundary-node-data-centers", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq ic network boundary-node-data-centers [OPTIONS]"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--format <text|json>"));
    assert!(stdout.contains("one official Dashboard v4 request"));
}

#[test]
fn binary_ic_daily_stats_help_smoke() {
    let output = run_icq(&["ic", "network", "daily-stats", "help"]);

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
    let output = run_icq(&["icrc", "account", "balance", "help"]);

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
    let output = run_icq(&["icrc", "ledger", "capabilities", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger capabilities [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--format <text|json>"));
}

#[test]
fn binary_icrc_allowance_help_smoke() {
    let output = run_icq(&["icrc", "account", "allowance", "help"]);

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
            &["icrc", "account", "transaction", "page", "help"][..],
            "Usage: icq icrc account transaction page [OPTIONS] <ledger-canister-id> <principal>",
            "--start <block-index>",
        ),
        (
            &["icrc", "account", "transaction", "list", "help"][..],
            "Usage: icq icrc account transaction list [OPTIONS] <ledger-canister-id> <principal>",
            "--sort <newest|oldest>",
        ),
        (
            &["icrc", "account", "transaction", "refresh", "help"][..],
            "Usage: icq icrc account transaction refresh [OPTIONS] <ledger-canister-id> <principal>",
            "--page-size <count>",
        ),
        (
            &["icrc", "account", "transaction", "cache", "status", "help"][..],
            "Usage: icq icrc account transaction cache status [OPTIONS] <ledger-canister-id> <principal>",
            "--source-endpoint <url>",
        ),
    ] {
        let output = run_icq(args);
        assert_success(&output);
        let stdout = stdout_text(&output);
        assert!(stdout.contains(usage), "missing {usage:?} in {stdout}");
        assert!(stdout.contains(option), "missing {option:?} in {stdout}");
        assert!(stdout.contains("--format <text|json>"));
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
            "--format",
            "json",
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
        &["nns", "proposal", "cache", "status", "--format", "json"],
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
    let output = run_icq(&["icrc", "ledger", "index", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger index [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--format <text|json>"));
}

#[test]
fn binary_icrc_transactions_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "transactions", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger transactions [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--start <index>"));
    assert!(stdout.contains("--limit <count>"));
    assert!(stdout.contains("--follow-archives"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--format <text|json>"));
}

#[test]
fn binary_icrc_block_types_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "block-types", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger block-types [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--format <text|json>"));
}

#[test]
fn binary_icrc_archives_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "archives", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc ledger archives [OPTIONS] <ledger-canister-id>"));
    assert!(stdout.contains("--from <canister-id>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--format <text|json>"));
}

#[test]
fn binary_icrc_tip_certificate_help_smoke() {
    let output = run_icq(&["icrc", "ledger", "tip-certificate", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("Usage: icq icrc ledger tip-certificate [OPTIONS] <ledger-canister-id>")
    );
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("--format <text|json>"));
}

#[test]
fn binary_sns_list_help_smoke() {
    let output = run_icq(&["sns", "list", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns list [OPTIONS]"));
    assert!(stdout.contains("--sort <id|name>"));
    assert!(stdout.contains("--verbose"));
}

#[test]
fn binary_sns_canister_list_help_smoke() {
    let output = run_icq(&["sns", "canister", "list", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq sns canister list [OPTIONS] <id|root-principal>"));
    assert!(stdout.contains("--source-endpoint <url>"));
    assert!(stdout.contains("update_canister_list=false"));
}

#[test]
fn binary_nns_topology_help_smoke() {
    let output = run_icq(&["nns", "topology", "help"]);

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

    let nns_status = run_icq_in_root(
        &root,
        &["nns", "proposal", "cache", "status", "--format", "json"],
    );
    assert_success(&nns_status);
    let nns_status: serde_json::Value =
        serde_json::from_str(&stdout_text(&nns_status)).expect("nns cache status json");
    assert_eq!(nns_status["found"], false);

    let sns_proposals = run_icq_in_root(
        &root,
        &["sns", "proposal", "cache", "list", "--format", "json"],
    );
    assert_success(&sns_proposals);
    let sns_proposals: serde_json::Value =
        serde_json::from_str(&stdout_text(&sns_proposals)).expect("sns proposals cache list json");
    assert_eq!(sns_proposals["cache_count"], 0);

    let sns_neurons = run_icq_in_root(
        &root,
        &["sns", "neuron", "cache", "list", "--format", "json"],
    );
    assert_success(&sns_neurons);
    let sns_neurons: serde_json::Value =
        serde_json::from_str(&stdout_text(&sns_neurons)).expect("sns neurons cache list json");
    assert_eq!(sns_neurons["cache_count"], 0);

    let _ = fs::remove_dir_all(root);
}
