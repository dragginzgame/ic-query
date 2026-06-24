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
        .env("ICQ_ICP_ROOT", root)
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

fn temp_icp_root(prefix: &str) -> std::path::PathBuf {
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
    assert!(stdout.contains("icrc"));
    assert!(stdout.contains("nns"));
    assert!(stdout.contains("sns"));
}

#[test]
fn binary_icrc_balance_help_smoke() {
    let output = run_icq(&["icrc", "balance", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq icrc balance [OPTIONS] <ledger-canister-id> <principal>"));
    assert!(stdout.contains("--subaccount <hex>"));
    assert!(stdout.contains("--source-endpoint <url>"));
}

#[test]
fn binary_icrc_allowance_help_smoke() {
    let output = run_icq(&["icrc", "allowance", "help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains(
        "Usage: icq icrc allowance [OPTIONS] <ledger-canister-id> <owner-principal> <spender-principal>"
    ));
    assert!(stdout.contains("--owner-subaccount <hex>"));
    assert!(stdout.contains("--spender-subaccount <hex>"));
    assert!(stdout.contains("--source-endpoint <url>"));
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
    let root = temp_icp_root("ic-query-cli-cache-json");
    fs::create_dir_all(&root).expect("create temp icp root");

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
        &["sns", "proposals", "cache", "list", "--format", "json"],
    );
    assert_success(&sns_proposals);
    let sns_proposals: serde_json::Value =
        serde_json::from_str(&stdout_text(&sns_proposals)).expect("sns proposals cache list json");
    assert_eq!(sns_proposals["cache_count"], 0);

    let sns_neurons = run_icq_in_root(
        &root,
        &["sns", "neurons", "cache", "list", "--format", "json"],
    );
    assert_success(&sns_neurons);
    let sns_neurons: serde_json::Value =
        serde_json::from_str(&stdout_text(&sns_neurons)).expect("sns neurons cache list json");
    assert_eq!(sns_neurons["cache_count"], 0);

    let _ = fs::remove_dir_all(root);
}
