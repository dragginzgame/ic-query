//! Process-level contracts that require no network access.

use std::process::{Command, Output};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_icq"))
        .args(arguments)
        .env("NO_COLOR", "1")
        .output()
        .expect("icq should start")
}

#[test]
fn bare_invocation_and_explicit_help_match() {
    let expected = run(&["--help"]);
    assert!(expected.status.success());
    assert!(expected.stderr.is_empty());
    let text = String::from_utf8_lossy(&expected.stdout);
    assert!(text.contains("Usage: icq"));
    assert!(text.contains("topology"));
    assert!(text.find("  help ").unwrap() < text.find("  topology ").unwrap());

    for arguments in [&[][..], &["help"][..]] {
        let output = run(arguments);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, expected.stdout);
    }
}

#[test]
fn version_identifies_the_cli_package() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 version"),
        format!("icq {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unsupported_operations_fail_without_success_output() {
    for arguments in [
        &["unsupported"][..],
        &["--unknown"][..],
        &["help", "unknown"][..],
    ] {
        let output = run(arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn topology_namespace_help_is_local_and_consistent() {
    let expected = run(&["topology", "--help"]);
    assert!(expected.status.success());
    assert!(expected.stderr.is_empty());
    let text = String::from_utf8_lossy(&expected.stdout);
    assert!(text.find("  canister ").unwrap() < text.find("  help ").unwrap());
    assert!(text.find("  help ").unwrap() < text.find("  subnet ").unwrap());
    for arguments in [
        &["topology"][..],
        &["topology", "help"][..],
        &["help", "topology"][..],
    ] {
        let output = run(arguments);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, expected.stdout);
    }
}

#[test]
fn report_commands_require_valid_principals_before_network_access() {
    for (command, placeholder) in [("canister", "CANISTER_ID"), ("subnet", "SUBNET_ID")] {
        for arguments in [
            &["topology", command, "show"][..],
            &["topology", command, "show", "invalid"][..],
        ] {
            let output = run(arguments);
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            assert!(!output.stderr.is_empty());
        }
        let help = run(&["topology", command, "show", "--help"]);
        assert!(help.status.success());
        assert!(help.stderr.is_empty());
        let text = String::from_utf8_lossy(&help.stdout);
        assert!(text.contains("--json"));
        assert!(text.contains(placeholder));
    }
}

#[test]
fn entity_namespace_help_is_local_and_consistent() {
    for entity in ["canister", "subnet"] {
        let expected = run(&["topology", entity, "--help"]);
        assert!(expected.status.success());
        assert!(expected.stderr.is_empty());
        let text = String::from_utf8_lossy(&expected.stdout);
        assert!(text.find("  help ").unwrap() < text.find("  show ").unwrap());
        if entity == "subnet" {
            assert!(text.find("  help ").unwrap() < text.find("  list ").unwrap());
            assert!(text.find("  list ").unwrap() < text.find("  metrics ").unwrap());
            assert!(text.find("  metrics ").unwrap() < text.find("  show ").unwrap());
        }
        for arguments in [
            &["topology", entity][..],
            &["topology", entity, "help"][..],
            &["help", "topology", entity][..],
        ] {
            let output = run(arguments);
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            assert_eq!(output.stdout, expected.stdout);
        }
    }
}

#[test]
fn subnet_list_help_and_argument_rejection_are_local() {
    let help = run(&["topology", "subnet", "list", "--help"]);
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--json"));
    for arguments in [
        &["topology", "subnet", "list", "unexpected"][..],
        &["topology", "subnet", "list", "--unknown"][..],
    ] {
        let output = run(arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn subnet_metrics_requires_a_typed_identity_and_discloses_its_scope() {
    let help = run(&["topology", "subnet", "metrics", "--help"]);
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("SUBNET_ID"));
    assert!(text.contains("--json"));
    assert!(text.contains("not capacity, rates or uptime"));
    for arguments in [
        &["topology", "subnet", "metrics"][..],
        &["topology", "subnet", "metrics", "invalid"][..],
        &["topology", "subnet", "metrics", "--unknown"][..],
    ] {
        let output = run(arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
