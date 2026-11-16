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
        if entity == "node" {
            assert!(text.find("  changes ").unwrap() < text.find("  help ").unwrap());
        }
        if entity == "subnet" {
            assert!(text.find("  activity ").unwrap() < text.find("  changes ").unwrap());
            assert!(text.find("  changes ").unwrap() < text.find("  configuration ").unwrap());
            assert!(text.find("  configuration ").unwrap() < text.find("  distribution ").unwrap());
            assert!(text.find("  distribution ").unwrap() < text.find("  help ").unwrap());
            assert!(text.find("  help ").unwrap() < text.find("  list ").unwrap());
            assert!(text.find("  list ").unwrap() < text.find("  membership ").unwrap());
            assert!(text.find("  membership ").unwrap() < text.find("  metrics ").unwrap());
            assert!(text.find("  metrics ").unwrap() < text.find("  nodes ").unwrap());
            assert!(text.find("  nodes ").unwrap() < text.find("  show ").unwrap());
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
fn distribution_help_and_invalid_arguments_are_local() {
    let help = run(&["topology", "subnet", "distribution", "--help"]);
    assert!(help.status.success() && help.stderr.is_empty());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("SUBNET_ID") && text.contains("--json"));
    assert!(text.contains("partial output exits 1"));
    for arguments in [
        &["topology", "subnet", "distribution"][..],
        &["topology", "subnet", "distribution", "invalid"][..],
        &["topology", "subnet", "distribution", "--unknown"][..],
    ] {
        let output = run(arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty() && !output.stderr.is_empty());
    }
}

#[test]
fn node_changes_help_and_invalid_intervals_are_local() {
    let help = run(&["topology", "node", "changes", "--help"]);
    assert!(help.status.success() && help.stderr.is_empty());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("not TEE readiness"));
    assert!(text.contains("NODE_ID") && text.contains("--json"));
    assert!(text.contains("[default: 10]"));
    for arguments in [
        &["topology", "node", "changes"][..],
        &["topology", "node", "changes", "invalid"][..],
        &["topology", "node", "changes", "--unknown"][..],
    ] {
        let output = run(arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
    for interval in ["0", "61", "18446744073709551615"] {
        let output = run(&[
            "topology",
            "node",
            "changes",
            "nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae",
            "--interval-seconds",
            interval,
            "--json",
        ]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty() && !output.stderr.is_empty());
    }
}

#[test]
fn subnet_changes_help_and_invalid_intervals_are_local() {
    let help = run(&["topology", "subnet", "changes", "--help"]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("not a maintenance event log"));
    assert!(text.contains("SUBNET_ID") && text.contains("--json"));
    assert!(text.contains("[default: 10]"));
    for arguments in [
        &["topology", "subnet", "changes"][..],
        &["topology", "subnet", "changes", "invalid"][..],
        &["topology", "subnet", "changes", "--unknown"][..],
    ] {
        let output = run(arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
    for interval in ["0", "61", "18446744073709551615"] {
        let output = run(&[
            "topology",
            "subnet",
            "changes",
            "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe",
            "--interval-seconds",
            interval,
            "--json",
        ]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn subnet_nodes_requires_one_typed_identity_and_discloses_partial_exit_status() {
    let help = run(&["topology", "subnet", "nodes", "--help"]);
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("SUBNET_ID"));
    assert!(text.contains("--json"));
    assert!(text.contains("partial output exits with status 1"));
    for arguments in [
        &["topology", "subnet", "nodes"][..],
        &["topology", "subnet", "nodes", "invalid"][..],
        &["topology", "subnet", "nodes", "--unknown"][..],
        &[
            "topology",
            "subnet",
            "nodes",
            "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe",
            "extra",
        ][..],
    ] {
        let output = run(arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
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

#[test]
fn cloud_engine_prices_namespace_and_rejections_are_local() {
    let expected = run(&["topology", "cloud-engine", "--help"]);
    assert!(expected.status.success() && expected.stderr.is_empty());
    let text = String::from_utf8_lossy(&expected.stdout);
    assert!(text.find("  available ").unwrap() < text.find("  help ").unwrap());
    assert!(text.find("  help ").unwrap() < text.find("  prices ").unwrap());
    for args in [
        &["topology", "cloud-engine"][..],
        &["topology", "cloud-engine", "help"][..],
        &["help", "topology", "cloud-engine"][..],
    ] {
        assert_eq!(run(args).stdout, expected.stdout);
    }
    let help = run(&["topology", "cloud-engine", "prices", "--help"]);
    assert!(help.status.success() && help.stderr.is_empty());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(
        text.contains("daily")
            && text.contains("--json")
            && text.contains("partial output exits 1")
    );
    for args in [
        &["topology", "cloud-engine", "prices", "unexpected-id"][..],
        &[
            "topology",
            "cloud-engine",
            "prices",
            "--method",
            "getApiDoc",
        ][..],
        &[
            "topology",
            "cloud-engine",
            "prices",
            "--endpoint",
            "https://example.com",
        ][..],
        &["topology", "cloud-engine", "available", "unexpected-id"][..],
        &[
            "topology",
            "cloud-engine",
            "available",
            "--rejoining",
            "private-engine",
        ][..],
        &[
            "topology",
            "cloud-engine",
            "available",
            "--start",
            "aaaaa-aa",
        ][..],
    ] {
        let output = run(args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty() && !output.stderr.is_empty());
    }
}

#[test]
fn available_help_declares_bounded_public_scope() {
    let help = run(&["topology", "cloud-engine", "available", "--help"]);
    assert!(help.status.success() && help.stderr.is_empty());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(
        text.contains("bounded pagination")
            && text.contains("--json")
            && text.contains("partial output exits 1")
    );
}

#[test]
fn available_page_budget_is_parsed_and_bounded_before_io() {
    for value in ["0", "101", "65535", "65536", "-1", "invalid"] {
        let output = run(&[
            "topology",
            "cloud-engine",
            "available",
            "--max-pages",
            value,
        ]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty() && !output.stderr.is_empty());
    }
    let output = run(&[
        "topology",
        "cloud-engine",
        "available",
        "--max-pages",
        "100",
        "--help",
    ]);
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("--max-pages") && help.contains("[default: 10]"));
}

#[test]
fn candidates_help_and_invalid_budgets_require_no_network() {
    let output = run(&["topology", "cloud-engine", "candidates", "--help"]);
    assert!(output.status.success() && output.stderr.is_empty());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("does not select a tariff")
            && text.contains("--max-pages")
            && text.contains("--json")
    );
    for value in ["0", "101", "-1", "65536", "invalid"] {
        let output = run(&[
            "topology",
            "cloud-engine",
            "candidates",
            "--max-pages",
            value,
        ]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty() && !output.stderr.is_empty());
    }
    for flag in ["--owner", "--reserve", "--quote", "--method"] {
        let output = run(&["topology", "cloud-engine", "candidates", flag]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn selected_engine_help_and_id_refusals_are_local() {
    let help = run(&["topology", "cloud-engine", "show", "--help"]);
    assert!(help.status.success() && help.stderr.is_empty());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("ENGINE_ID") && text.contains("--json") && text.contains("not liveness"));
    let missing = run(&["topology", "cloud-engine", "show"]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(missing.stdout.is_empty());
    for id in ["", &"x".repeat(257), &"é".repeat(129)] {
        let output = run(&["topology", "cloud-engine", "show", id, "--json"]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("1–256 UTF-8 bytes"));
    }
    for flag in [
        "--owner",
        "--method",
        "--endpoint",
        "--max-pages",
        "--update",
    ] {
        let output = run(&["topology", "cloud-engine", "show", "engine-x", flag]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn maintenance_help_and_pre_io_refusals() {
    let help = run(&["topology", "subnet", "maintenance", "--help"]);
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(
        text.contains("--expected-version")
            && text.contains("--interval-seconds")
            && text.contains("--json")
    );
    let subnet = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";
    for interval in ["0", "61"] {
        let out = run(&[
            "topology",
            "subnet",
            "maintenance",
            subnet,
            "--interval-seconds",
            interval,
        ]);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
    }
    let out = run(&[
        "topology",
        "subnet",
        "maintenance",
        subnet,
        "--expected-version",
        &"é".repeat(129),
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let out = run(&["topology", "subnet", "maintenance", "not-principal"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn compare_help_and_duplicate_selection_fail_before_io() {
    let help = run(&["topology", "subnet", "compare", "--help"]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(
        text.contains("LEFT_SUBNET_ID")
            && text.contains("RIGHT_SUBNET_ID")
            && text.contains("--json")
            && text.contains("--overlap-details")
    );
    let subnet = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";
    let duplicate = run(&["topology", "subnet", "compare", subnet, subnet, "--json"]);
    assert_eq!(duplicate.status.code(), Some(1));
    assert!(duplicate.stdout.is_empty());
    let conflicting = run(&[
        "topology",
        "subnet",
        "compare",
        subnet,
        subnet,
        "--json",
        "--overlap-details",
    ]);
    assert_eq!(conflicting.status.code(), Some(2));
    assert!(conflicting.stdout.is_empty());
    for args in [
        vec!["topology", "subnet", "compare", subnet],
        vec!["topology", "subnet", "compare", subnet, "not-principal"],
    ] {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
    }
}

#[test]
fn placements_help_and_invalid_selections_are_offline() {
    let help = run(&["topology", "canister", "placements", "--help"]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("CANISTER_ID") && text.contains("--json"));
    for args in [
        vec!["topology", "canister", "placements"],
        vec!["topology", "canister", "placements", "not-principal"],
    ] {
        let result = run(&args);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
    let id = "ryjl3-tyaaa-aaaaa-aaaba-cai";
    let duplicate = run(&["topology", "canister", "placements", id, id, "--json"]);
    assert_eq!(duplicate.status.code(), Some(1));
    assert!(duplicate.stdout.is_empty());
    let mut excess = vec!["topology", "canister", "placements"];
    excess.extend(std::iter::repeat_n(id, 33));
    let result = run(&excess);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
}

#[test]
fn settings_help_and_invalid_selection_are_local() {
    let output = run(&["topology", "subnet", "settings", "--help"]);
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("SUBNET_ID") && text.contains("--json"));
    for args in [
        vec!["topology", "subnet", "settings"],
        vec!["topology", "subnet", "settings", "invalid"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn settings_comparison_flag_is_local_and_duplicate_selection_reads_nothing() {
    let help = run(&["topology", "subnet", "compare", "--help"]);
    assert!(String::from_utf8_lossy(&help.stdout).contains("--settings"));
    for format in ["--json", "--overlap-details"] {
        let output = run(&[
            "topology",
            "subnet",
            "compare",
            "aaaaa-aa",
            "aaaaa-aa",
            "--settings",
            format,
        ]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("two different"));
    }
}

#[test]
fn progress_help_and_admission_are_offline() {
    let help = run(&["topology", "subnet", "progress", "--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--interval-seconds"));
    for interval in ["0", "61"] {
        let result = run(&[
            "topology",
            "subnet",
            "progress",
            "aaaaa-aa",
            "--interval-seconds",
            interval,
        ]);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
}
