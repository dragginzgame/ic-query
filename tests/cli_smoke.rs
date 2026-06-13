use std::process::{Command, Output};

fn run_icq(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_icq"))
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

#[test]
fn binary_top_level_help_smoke() {
    let output = run_icq(&["help"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage: icq [OPTIONS] [COMMAND]"));
    assert!(stdout.contains("nns"));
    assert!(stdout.contains("sns"));
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
