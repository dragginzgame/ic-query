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
    assert!(text.find("  help ").unwrap() < text.find("  node ").unwrap());
    assert!(text.find("  node ").unwrap() < text.find("  subnet ").unwrap());
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
    for (command, placeholder) in [
        ("canister", "CANISTER_ID"),
        ("node", "NODE_ID"),
        ("subnet", "SUBNET_ID"),
    ] {
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
        if command == "node" {
            assert!(text.contains("not verified attestation"));
        }
    }
}

#[test]
fn entity_namespace_help_is_local_and_consistent() {
    for entity in ["canister", "node", "subnet"] {
        let expected = run(&["topology", entity, "--help"]);
        assert!(expected.status.success());
        assert!(expected.stderr.is_empty());
        let text = String::from_utf8_lossy(&expected.stdout);
        assert!(text.find("  help ").unwrap() < text.find("  show ").unwrap());
        if entity == "subnet" {
            assert!(text.find("  activity ").unwrap() < text.find("  configuration ").unwrap());
            assert!(text.find("  configuration ").unwrap() < text.find("  help ").unwrap());
            assert!(text.find("  help ").unwrap() < text.find("  list ").unwrap());
            assert!(text.find("  list ").unwrap() < text.find("  membership ").unwrap());
            assert!(text.find("  membership ").unwrap() < text.find("  metrics ").unwrap());
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

#[test]
fn subnet_configuration_requires_an_identity_and_discloses_configured_state() {
    let help = run(&["topology", "subnet", "configuration", "--help"]);
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("SUBNET_ID"));
    assert!(text.contains("--json"));
    assert!(text.contains("not running state"));
    for arguments in [
        &["topology", "subnet", "configuration"][..],
        &["topology", "subnet", "configuration", "invalid"][..],
        &["topology", "subnet", "configuration", "--unknown"][..],
    ] {
        let output = run(arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn subnet_membership_help_and_invalid_arguments_remain_local() {
    let help = run(&["topology", "subnet", "membership", "--help"]);
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("SUBNET_ID"));
    assert!(text.contains("--json"));
    assert!(text.contains("not an atomic snapshot"));
    for arguments in [
        &["topology", "subnet", "membership"][..],
        &["topology", "subnet", "membership", "invalid"][..],
        &["topology", "subnet", "membership", "--unknown"][..],
    ] {
        let output = run(arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn subnet_activity_help_and_invalid_inputs_do_not_access_the_network() {
    let help = run(&["topology", "subnet", "activity", "--help"]);
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("not uptime or throughput"));
    assert!(text.contains("--interval-seconds"));
    assert!(text.contains("[default: 10]"));
    assert!(text.contains("--json"));
    let id = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";
    for arguments in [
        vec!["topology", "subnet", "activity"],
        vec!["topology", "subnet", "activity", "invalid"],
        vec![
            "topology",
            "subnet",
            "activity",
            id,
            "--interval-seconds",
            "oops",
        ],
        vec!["topology", "subnet", "activity", id, "--unknown"],
    ] {
        let output = run(&arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    for interval in ["0", "61", "18446744073709551615"] {
        let output = run(&[
            "topology",
            "subnet",
            "activity",
            id,
            "--interval-seconds",
            interval,
            "--json",
        ]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("between 1 and 60 seconds"));
    }
}
