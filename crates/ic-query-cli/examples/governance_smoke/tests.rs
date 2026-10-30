use super::*;
use command::{CommandFailure, CommandOptions, StderrMode};
use ic_build::artifacts::WasmBuildError;
use ic_host_artifacts::{
    artifact::{ArtifactError, Sha256Digest},
    wasm::{CustomSectionEncodingError, InspectionError, InspectionLimits, inspect},
};
use ic_host_fs::{
    durable::{NamedWriteError, PublicationMode, WriteOptions, write_with},
    testing::FixtureDirectory,
};
use ic_host_process::tool::{ExecutionFailure, ToolError};
use protocol::Probe;
use std::{
    fmt,
    os::unix::{
        ffi::OsStringExt,
        fs::{PermissionsExt, symlink},
    },
    time::{Duration, Instant},
};

#[derive(Debug)]
struct FormattedCause;

impl fmt::Display for FormattedCause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.pad("cause")
    }
}

impl Error for FormattedCause {}

#[test]
fn command_failure_preserves_display_flags_and_direct_erased_source() {
    let failure = CommandFailure {
        source: Box::new(FormattedCause),
        stdout: Some(b"stdout".to_vec()),
        stderr: Some(b"stderr".to_vec()),
        cleanup_errors: Vec::new(),
        exited: true,
        interrupted: Some(libc::SIGTERM),
    };
    assert_eq!(failure.to_string(), "cause");
    assert_eq!(
        format!("{failure:>9.3}"),
        format!("{:>9.3}", failure.source)
    );
    assert_eq!(
        format!("{failure:*^11}"),
        format!("{:*^11}", failure.source)
    );
    let source = failure.source().expect("original cause");
    assert!(source.downcast_ref::<FormattedCause>().is_some());
    assert!(std::ptr::eq(source, failure.source.as_ref()));
    assert_eq!(failure.stdout.as_deref(), Some(b"stdout".as_slice()));
    assert_eq!(failure.stderr.as_deref(), Some(b"stderr".as_slice()));
    assert!(failure.exited);
    assert_eq!(failure.interrupted, Some(libc::SIGTERM));
}

fn payload(kind: &str) -> Value {
    json!({"status":"ok", "report":{
        "schema_version":1, "network":"ic", "governance_canister_id":GOVERNANCE,
        "fetched_at":"2026-09-15T12:00:00Z",
        "source":{"source_transport":"replicated_inter_canister_call", "collector_canister_id":"aaaaa-aa"},
        kind:{"raw_amount":10_000},
    }})
}

#[test]
fn ic_tool_admission_uses_consumer_make_selection_and_preserves_refusal() {
    let fixture = FixtureDirectory::new(&env::temp_dir(), "governance-ic-admission").unwrap();
    let root = fixture.path().join("consumer with spaces");
    fs::create_dir(&root).unwrap();
    fs::write(
        root.join("Makefile"),
        ".PHONY: unexpected ic-tools-check\nunexpected:\n\t@exit 73\nic-tools-check:\n\t@printf checked > admitted\n\t@test ! -f refuse\n",
    )
    .unwrap();
    build::check_ic_tools(&root).unwrap();
    assert_eq!(fs::read(root.join("admitted")).unwrap(), b"checked");
    fs::write(root.join("refuse"), b"").unwrap();
    let error = build::check_ic_tools(&root).unwrap_err();
    assert!(error.downcast_ref::<CommandFailure>().is_some());
}

#[test]
fn actions_are_admitted_before_effects_and_help_commands_are_sorted() {
    for (args, expected) in [
        (vec!["build"], Action::Build),
        (vec!["build-wasm"], Action::BuildWasm),
        (vec!["bundle"], Action::Bundle),
        (
            vec!["inspect-wasm", "probe.wasm"],
            Action::InspectWasm {
                path: "probe.wasm".into(),
            },
        ),
        (vec!["local"], Action::Local),
        (
            vec!["verify-mainnet", "--canister", "aaaaa-aa"],
            Action::VerifyMainnet {
                canister: "aaaaa-aa".into(),
            },
        ),
    ] {
        assert_eq!(
            parse(args.into_iter().map(OsString::from)).unwrap(),
            Some(expected)
        );
    }
    for args in [
        vec!["verify-mainnet"],
        vec!["verify-mainnet", "--canister", "invalid"],
        vec!["local", "--canister", "aaaaa-aa"],
        vec!["build", "extra"],
        vec!["inspect-wasm"],
    ] {
        assert!(parse(args.into_iter().map(OsString::from)).is_err());
    }
    let mut command = ic_query_cli::prepare_command_tree(Action::command());
    let help = command.render_help().to_string();
    let commands = help
        .split_once("Commands:\n")
        .unwrap()
        .1
        .split("\n\n")
        .next()
        .unwrap()
        .lines()
        .map(|line| line.split_whitespace().next().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        [
            "build",
            "build-wasm",
            "bundle",
            "help",
            "inspect-wasm",
            "local",
            "verify-mainnet"
        ]
    );
    for args in [vec![], vec!["help"], vec!["--help"], vec!["-h"]] {
        let error = command
            .clone()
            .try_get_matches_from(std::iter::once("governance-smoke").chain(args))
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        ));
        assert_eq!(error.to_string(), help);
    }
    let error = command
        .try_get_matches_from(["governance-smoke", "help", "inspect-wasm"])
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::DisplayHelp);
    assert!(error.to_string().contains("<path>"));
}

#[test]
fn canister_argument_preserves_admitted_text() {
    assert_eq!(
        parse(["verify-mainnet", "--canister", "AAAAA-AA"].map(OsString::from)).unwrap(),
        Some(Action::VerifyMainnet {
            canister: "AAAAA-AA".into()
        })
    );
}

#[test]
fn wasm_argument_preserves_non_utf8_paths() {
    let path = OsString::from_vec(b"probe-\xff.wasm".to_vec());
    assert_eq!(
        parse([OsString::from("inspect-wasm"), path.clone()]).unwrap(),
        Some(Action::InspectWasm { path: path.into() })
    );
    assert!(
        parse([
            OsString::from("verify-mainnet"),
            OsString::from("--canister"),
            OsString::from_vec(vec![0xff])
        ])
        .is_err()
    );
}

#[test]
fn source_fingerprint_covers_inventory_independent_of_checkout_location() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let relocated = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    for path in [
        "tests/canister/probe-wire.json",
        "rust-toolchain.toml",
        "crates/ic-query/src/lib.rs",
        "crates/ic-query/Cargo.toml",
        "crates/ic-query-cli/examples/governance_smoke/main.rs",
        "crates/ic-query-cli/Cargo.toml",
        "Cargo.toml",
        "Cargo.lock",
    ] {
        for root in [directory.path(), relocated.path()] {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"fixture\n").unwrap();
        }
    }
    let source = directory.path().join("crates/ic-query/src/large.rs");
    fs::write(&source, vec![b'x'; 65_537]).unwrap();
    fs::write(
        relocated.path().join("crates/ic-query/src/large.rs"),
        vec![b'x'; 65_537],
    )
    .unwrap();
    let expected = build::source_digest(directory.path()).unwrap();
    assert_eq!(build::source_digest(relocated.path()).unwrap(), expected);
    fs::write(&source, vec![b'y'; 65_537]).unwrap();
    assert_ne!(build::source_digest(directory.path()).unwrap(), expected);
    fs::write(&source, vec![b'x'; 65_537]).unwrap();
    let guide = directory.path().join("crates/ic-query/README.md");
    fs::write(&guide, b"guide").unwrap();
    assert_ne!(build::source_digest(directory.path()).unwrap(), expected);
    fs::remove_file(guide).unwrap();
    let empty = directory.path().join("crates/ic-query/empty");
    fs::create_dir(&empty).unwrap();
    assert_ne!(build::source_digest(directory.path()).unwrap(), expected);
    fs::remove_dir(empty).unwrap();
    assert_eq!(build::source_digest(directory.path()).unwrap(), expected);
    fs::rename(&source, source.with_file_name("renamed.rs")).unwrap();
    assert_ne!(build::source_digest(directory.path()).unwrap(), expected);
    fs::rename(source.with_file_name("renamed.rs"), source).unwrap();
    assert_eq!(build::source_digest(directory.path()).unwrap(), expected);
    fs::remove_file(directory.path().join("Cargo.lock")).unwrap();
    assert_eq!(
        build::source_digest(directory.path())
            .unwrap_err()
            .downcast_ref::<io::Error>()
            .unwrap()
            .kind(),
        io::ErrorKind::NotFound
    );
}

fn probe_workspace() -> FixtureDirectory {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    fs::create_dir(directory.path().join(".cargo")).unwrap();
    fs::write(
        directory.path().join(".cargo/config.toml"),
        "[build]\ntarget-dir = 'configured target'\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("Cargo.toml"),
        "[package]\nname = 'ic-query'\nversion = '0.0.0'\nedition = '2024'\n\
         [features]\ncanister = []\n\
         [[example]]\nname = 'governance_probe'\npath = 'probe.rs'\ncrate-type = ['cdylib']\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("Cargo.lock"),
        "version = 4\n[[package]]\nname = 'ic-query'\nversion = '0.0.0'\n",
    )
    .unwrap();
    fs::write(directory.path().join("probe.rs"), "pub fn probe() {}\n").unwrap();
    directory
}

#[test]
fn probe_inputs_bind_configured_output_source_and_cdylib_example() {
    let directory = probe_workspace();
    let target = build::target_directory(directory.path()).unwrap();
    assert_eq!(target, directory.path().join("configured target"));
    let spec = build::probe_spec(directory.path(), &target);
    let resolve = || build::resolve_probe_inputs(&spec, Duration::from_secs(10)).unwrap();
    let before = resolve();
    assert_eq!(
        before.artifact_paths().unwrap(),
        [target.join("wasm32-unknown-unknown/release/examples/governance_probe.wasm")]
    );
    let unchanged = resolve();
    assert_eq!(before.fingerprint(), unchanged.fingerprint());
    assert_eq!(before.validation_digest(), unchanged.validation_digest());
    fs::write(directory.path().join("probe.rs"), "pub fn changed() {}\n").unwrap();
    let changed = resolve();
    assert_ne!(before.fingerprint(), changed.fingerprint());
    assert_ne!(before.validation_digest(), changed.validation_digest());

    let manifest = directory.path().join("Cargo.toml");
    let original = fs::read_to_string(&manifest).unwrap();
    for invalid in [
        original.replace("['cdylib']", "['rlib']"),
        original.replace("name = 'governance_probe'", "name = 'another_probe'"),
    ] {
        fs::write(&manifest, invalid).unwrap();
        assert!(matches!(
            resolve().artifact_paths(),
            Err(WasmBuildError::InvalidSpec { .. })
        ));
    }
}

#[test]
fn probe_inputs_refuse_oversized_configuration() {
    let directory = probe_workspace();
    let config = directory.path().join(".cargo/config.toml");
    fs::write(&config, " ".repeat(1024 * 1024 + 1)).unwrap();
    let spec = build::probe_spec(directory.path(), &directory.path().join("target"));
    let error = build::resolve_probe_inputs(&spec, Duration::from_secs(10)).unwrap_err();
    assert!(
        matches!(error.downcast_ref::<WasmBuildError>(),
        Some(WasmBuildError::InvalidCargoConfiguration { path, .. }) if path == &config),
        "{error:?}"
    );
}

#[test]
fn probe_input_deadline_retains_original_process_evidence() {
    let directory = probe_workspace();
    let program = directory.path().join("stalled-cargo");
    fs::write(
        &program,
        "#!/bin/sh\nprintf 'original probe output'\nexec sleep 5\n",
    )
    .unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    let spec = build::probe_spec(directory.path(), &directory.path().join("target"))
        .with_cargo_program(program);
    let error = build::resolve_probe_inputs(&spec, Duration::from_secs(1)).unwrap_err();
    let WasmBuildError::Io { source, .. } = error.downcast_ref().unwrap() else {
        panic!("expected retained probe failure: {error}");
    };
    let Some(ToolError::Execution(error)) = source.get_ref().unwrap().downcast_ref() else {
        panic!("expected original Host failure: {error}");
    };
    assert!(matches!(error.failure, ExecutionFailure::TimedOut));
    assert_eq!(error.evidence.stdout, b"original probe output");
    assert!(error.evidence.status.is_some());
}

#[test]
fn persistent_identity_selection_resolves_missing_paths_and_symlinks() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let root = directory.path();
    let project = root.join("tests/canister");
    let target = root.join("configured target");
    symlink(root.join("target"), root.join("output alias")).unwrap();
    for selected in [
        PathBuf::new(),
        root.join("target/identity"),
        root.join("output alias/identity"),
        target.join("identity"),
        project.join(".icp/identity"),
    ] {
        assert!(identity_home(root, &project, &target, &selected).is_err());
    }
    identity_home(
        root,
        &project,
        &target,
        Path::new("persistent identity home"),
    )
    .unwrap();
    assert_eq!(
        fs::read_dir(root).unwrap().count(),
        1,
        "admission creates no state"
    );
    let command = icp_command(
        root,
        &target,
        &["network", "status", "local"],
        Path::new("persistent identity home"),
    )
    .unwrap();
    assert_eq!(command.get_program(), root.join(".tools/ic/bin/icp"));
    assert_eq!(command.get_current_dir(), Some(root));
    let vars: std::collections::BTreeMap<_, _> = command.get_envs().collect();
    assert_eq!(
        vars[std::ffi::OsStr::new("ICP_HOME")],
        Some(std::ffi::OsStr::new("persistent identity home"))
    );
    for name in ["ICP_NETWORK", "ICP_ENVIRONMENT", "ICP_PROJECT_ROOT"] {
        assert_eq!(vars[std::ffi::OsStr::new(name)], None);
    }
}

#[test]
fn protocol_metadata_matches_the_owned_wire_contract() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/canister/probe-wire.json");
    let metadata: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(metadata["schema_version"], 1);
    assert_eq!(metadata["codec"], "ic-wire");
    assert_eq!(metadata["method"], "report");
    assert_eq!(metadata["method_id"], protocol::REPORT_METHOD);
    assert_eq!(metadata["request_bytes"], protocol::REQUEST_LIMITS.bytes);
    assert_eq!(metadata["response_bytes"], protocol::RESPONSE_LIMITS.bytes);
    let request = protocol::encode_request("economics").unwrap();
    assert_eq!(request, b"IW\x01\0\0\0\0\0\0\0\0\x09economics");
    assert_eq!(
        protocol::decode_reply(b"IW\x01\0\0\0\0\0\0\0\x01\x02{}").unwrap(),
        "{}"
    );
    let kind: String =
        ic_wire::native::decode_request(1, &request, protocol::REQUEST_LIMITS).unwrap();
    assert_eq!(kind, "economics");
}

#[test]
fn wire_admission_preserves_large_text_and_raw_numbers() {
    let text = format!(
        "{{\"raw_amount\":184467440737095516160,\"padding\":\"{}\"}}",
        "x".repeat(1024 * 1024)
    );
    let decoded = protocol::decode_reply(
        &ic_wire::native::encode_response(
            protocol::REPORT_METHOD,
            &text,
            protocol::RESPONSE_LIMITS,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(decoded, text);
    let json: Value = serde_json::from_str(&decoded).unwrap();
    assert_eq!(json["raw_amount"].to_string(), "184467440737095516160");
}

#[test]
fn wire_admission_rejects_wrong_identity_types_trailing_and_truncated_frames() {
    let mut trailing =
        ic_wire::native::encode_response(1, "{}", protocol::RESPONSE_LIMITS).unwrap();
    trailing.push(0);
    let mut truncated =
        ic_wire::native::encode_response(1, "payload", protocol::RESPONSE_LIMITS).unwrap();
    truncated.pop();
    for bytes in [
        ic_wire::native::encode_response(2, "{}", protocol::RESPONSE_LIMITS).unwrap(),
        protocol::encode_request("economics").unwrap(),
        ic_wire::native::encode_response(1, &7_u64, protocol::RESPONSE_LIMITS).unwrap(),
        ic_wire::native::encode_response(
            1,
            &("{}", "{}"),
            ic_wire::Limits {
                nodes: 16,
                ..protocol::RESPONSE_LIMITS
            },
        )
        .unwrap(),
        trailing,
        truncated,
        b"invalid frame".to_vec(),
    ] {
        let error = protocol::decode_reply(&bytes).unwrap_err();
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::InvalidData
        );
    }
}

#[test]
fn rejected_replies_remain_private_and_cannot_be_replaced() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    for (name, bytes) in [
        ("invalid", b"not Wire".to_vec()),
        ("oversized", vec![0; 2 * 1024 * 1024 + 1]),
    ] {
        let path = directory.path().join(name);
        assert!(protocol::publish_reply(&path, &bytes).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
        let error = protocol::publish_reply(
            &path,
            &ic_wire::native::encode_response(
                protocol::REPORT_METHOD,
                "{}",
                protocol::RESPONSE_LIMITS,
            )
            .unwrap(),
        )
        .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<NamedWriteError<io::Error>>(),
            Some(NamedWriteError::BeforePublication { source, cleanup_error: None })
                if source.kind() == io::ErrorKind::AlreadyExists
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn trust_policy_refuses_remote_local_roots_and_unclean_endpoints() {
    for (environment, endpoint) in [
        ("local", "http://example.com"),
        ("local", "https://localhost"),
        ("mainnet-smoke", "http://example.com"),
        ("mainnet-smoke", "https://user:secret@example.com"),
        ("local", "http://127.0.0.1/?network=other"),
    ] {
        assert!(protocol::agent(environment, endpoint).is_err());
    }
    for (environment, endpoint) in [
        ("local", "http://127.0.0.1:1234/"),
        ("local", "http://[::1]:1234/"),
        ("mainnet-smoke", "https://icp-api.io/"),
    ] {
        protocol::agent(environment, endpoint).unwrap();
    }
}

#[test]
fn report_admission_checks_identity_transport_and_canonical_time() {
    let good = payload("economics");
    assert_eq!(
        attempt::validate_report("economics", good.clone(), "aaaaa-aa").unwrap(),
        good["report"]
    );
    for timestamp in [
        "1970-01-01T00:00:00Z",
        "2000-02-29T23:59:59Z",
        "9999-12-31T23:59:59Z",
    ] {
        let mut report = good.clone();
        report["report"]["fetched_at"] = timestamp.into();
        assert_eq!(
            attempt::validate_report("economics", report.clone(), "aaaaa-aa").unwrap(),
            report["report"]
        );
    }
    for (key, value) in [
        ("schema_version", json!(2)),
        ("network", json!("local")),
        ("governance_canister_id", json!("aaaaa-aa")),
        ("source", json!({"source_transport":"replica_query"})),
        ("fetched_at", json!("2026-02-30T12:00:00Z")),
        ("fetched_at", json!("2026-09-15T12:00:00+00:00")),
        ("fetched_at", json!("2026-09-15T12:00:00.001Z")),
        ("fetched_at", json!("2026-9-15T12:00:00Z")),
        ("fetched_at", json!("2026-09-15T12:00:60Z")),
        ("fetched_at", json!("1969-12-31T23:59:59Z")),
        ("fetched_at", json!("10000-01-01T00:00:00Z")),
    ] {
        let mut bad = good.clone();
        bad["report"][key] = value;
        assert!(attempt::validate_report("economics", bad, "aaaaa-aa").is_err());
    }
    assert!(attempt::validate_report("metrics", good.clone(), "aaaaa-aa").is_err());
    assert!(attempt::validate_report("economics", good, "ryjl3-tyaaa-aaaaa-aaaba-cai").is_err());
    assert!(
        attempt::validate_report(
            "economics",
            json!({"status":"error", "error":"fixture"}),
            "aaaaa-aa"
        )
        .is_err()
    );
}

struct FixtureProbe {
    hash: String,
    changed: bool,
    calls: usize,
    invalid_kind: Option<&'static str>,
}

impl Probe for FixtureProbe {
    fn module_hash(&mut self) -> Result<String> {
        self.calls += 1;
        Ok(if self.changed && self.calls > 1 {
            "changed".into()
        } else {
            self.hash.clone()
        })
    }
    fn metadata(&mut self, name: &str) -> Result<Vec<u8>> {
        Ok(if name == "ic-query:protocol" {
            b"{\"schema_version\":1,\"codec\":\"ic-wire\"}\n".to_vec()
        } else {
            b"{\"schema_version\":1}".to_vec()
        })
    }
    fn report(&mut self, kind: &str) -> Result<Vec<u8>> {
        if self.invalid_kind == Some(kind) {
            return Ok(b"invalid Wire".to_vec());
        }
        Ok(ic_wire::native::encode_response(
            protocol::REPORT_METHOD,
            &serde_json::to_string(&payload(kind))?,
            protocol::RESPONSE_LIMITS,
        )?)
    }
}

fn verify_fixture(
    directory: &FixtureDirectory,
    changed: bool,
    invalid_kind: Option<&'static str>,
    bad_wasm: bool,
) -> (Result<()>, Value) {
    let bytes = if bad_wasm {
        b"not Wasm".as_slice()
    } else {
        b"\0asm\x01\0\0\0".as_slice()
    };
    let wasm = directory.path().join("probe.wasm");
    fs::write(&wasm, bytes).unwrap();
    let mut probe = FixtureProbe {
        hash: Sha256Digest::compute(bytes).to_string(),
        changed,
        calls: 0,
        invalid_kind,
    };
    let mut receipt = fixture_receipt();
    let output = directory.path().join("receipt.json");
    let result = attempt::verify(
        &mut probe,
        "aaaaa-aa",
        &wasm,
        b"{\"schema_version\":1,\"codec\":\"ic-wire\"}\n",
        &output,
        &mut receipt,
        |receipt| attempt::save_receipt(&output, receipt),
    );
    (result, serde_json::to_value(receipt).unwrap())
}

#[test]
fn verification_preserves_reports_raw_replies_and_final_module_check() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let (result, receipt) = verify_fixture(&directory, false, None, false);
    result.unwrap();
    assert_eq!(receipt["reports"].as_object().unwrap().len(), 4);
    assert_eq!(receipt["module_hash_before"], receipt["module_hash_after"]);
    for kind in receipt::ReportKind::ALL {
        let kind = kind.as_str();
        let bytes = fs::read(directory.path().join(format!("{kind}.wire"))).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&protocol::decode_reply(&bytes).unwrap()).unwrap(),
            payload(kind)
        );
    }
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let (result, receipt) = verify_fixture(&directory, true, None, false);
    assert!(result.is_err());
    assert_eq!(receipt["reports"].as_object().unwrap().len(), 4);
}

#[test]
fn invalid_reply_retains_prior_reports_and_a_checkpoint_before_admission() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let (result, receipt) = verify_fixture(&directory, false, Some("metrics"), false);
    assert!(result.is_err());
    assert!(receipt["reports"]["economics"].get("report").is_some());
    assert_eq!(
        receipt["reports"]["metrics"],
        json!({"reply_file":"metrics.wire"})
    );
    assert_eq!(
        fs::read(directory.path().join("metrics.wire")).unwrap(),
        b"invalid Wire"
    );
    let persisted: Value =
        serde_json::from_slice(&fs::read(directory.path().join("receipt.json")).unwrap()).unwrap();
    assert_eq!(persisted["phase"], "collecting_metrics");
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let (result, receipt) = verify_fixture(&directory, false, None, true);
    assert!(result.is_err());
    assert!(receipt.get("reports").is_none());
}

#[derive(Default)]
struct Operations {
    running: bool,
    startup_error: bool,
    discovery_error: bool,
    cleanup_error: bool,
    storage_error: Option<&'static str>,
    fail_after_start: bool,
    saves: Vec<Value>,
    calls: Vec<Vec<String>>,
    verified: bool,
    verify_error: bool,
    command_evidence: bool,
}

fn exit_failure() -> Box<dyn Error> {
    Box::new(CommandFailure {
        source: invalid("not running"),
        exited: true,
        stdout: None,
        stderr: None,
        cleanup_errors: vec![],
        interrupted: None,
    })
}

impl AttemptOperations for Operations {
    fn icp(&mut self, args: &[&str], options: &CommandOptions<'_>) -> Result<String> {
        self.calls
            .push(args.iter().map(|arg| (*arg).to_string()).collect());
        match args {
            ["network", "status", "local"] => {
                if self.running {
                    Ok("running".into())
                } else if self.discovery_error {
                    Err(invalid("status timeout"))
                } else {
                    Err(exit_failure())
                }
            }
            ["network", "start", "local", "--background"] => {
                assert!(options.handoff);
                assert_eq!(options.timeout, Duration::from_secs(900));
                if self.command_evidence {
                    Err(Box::new(CommandFailure {
                        source: invalid("startup deadline"),
                        stdout: Some(vec![255]),
                        stderr: Some(b"command error".to_vec()),
                        cleanup_errors: vec![ic_host_process::child::CleanupError {
                            status: None,
                            term_error: None,
                            group_error: Some(io::Error::other("refused")),
                            kill_error: None,
                            wait_error: None,
                        }],
                        exited: false,
                        interrupted: None,
                    }))
                } else if self.startup_error {
                    Err(invalid("startup failed"))
                } else {
                    Ok(String::new())
                }
            }
            ["network", "status", "local", "--json"] => {
                Ok(json!({"api_url":"http://127.0.0.1:1234/"}).to_string())
            }
            ["deploy", "-e", "local", "--json"] => Ok(
                json!({"canisters":[{"name":"governance-probe", "canister_id":"aaaaa-aa"}]})
                    .to_string(),
            ),
            ["network", "stop", "local"] => {
                assert!(options.cleanup);
                assert_eq!(options.timeout, Duration::from_secs(30));
                if self.cleanup_error {
                    Err(invalid("stop failed"))
                } else {
                    Ok(String::new())
                }
            }
            ["build", "-e", "mainnet-smoke"] => Ok(String::new()),
            _ => panic!("unexpected fixture command {args:?}"),
        }
    }
    fn publish(&mut self, receipt: &Receipt) -> Result<()> {
        let receipt = serde_json::to_value(receipt).unwrap();
        if self
            .storage_error
            .is_some_and(|phase| receipt["phase"] == phase)
            || self.fail_after_start
                && receipt["status"] == "failed"
                && receipt["phase"] != "finished"
        {
            return Err(invalid("receipt unavailable"));
        }
        self.saves.push(receipt.clone());
        Ok(())
    }
    fn verify(&mut self, _environment: &str, canister: &str, receipt: &mut Receipt) -> Result<()> {
        assert_eq!(canister, "aaaaa-aa");
        assert_eq!(receipt.status, receipt::Status::Running);
        self.verified = true;
        if self.verify_error {
            Err(invalid("verification failed"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn failed_receipts_keep_command_bytes_and_cleanup_errors_separate_from_the_primary() {
    let mut operations = Operations {
        command_evidence: true,
        cleanup_error: true,
        fail_after_start: true,
        ..Default::default()
    };
    let (result, receipt) = run_fixture(&mut operations);
    let error = result.unwrap_err();
    let command_error = error.downcast_ref::<CommandFailure>().unwrap();
    assert_eq!(command_error.source.to_string(), "startup deadline");
    assert_eq!(
        command_error.cleanup_errors[0]
            .group_error
            .as_ref()
            .unwrap()
            .kind(),
        io::ErrorKind::Other
    );
    assert_eq!(receipt["error"], "startup deadline");
    assert_eq!(receipt["command_stdout"], "\u{fffd}");
    assert_eq!(receipt["command_stderr"], "command error");
    assert_eq!(
        receipt["command_cleanup_errors"],
        json!(["child cleanup failed; group signal: refused"])
    );
    assert_eq!(receipt["cleanup_error"], "stop failed");
    assert_eq!(
        operations.saves.last().unwrap(),
        &receipt,
        "storage recovery retains every failure"
    );
}

#[test]
fn metadata_sections_preserve_exact_contents_and_wasm_framing() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let path = directory.path().join("probe.wasm");
    let mut bytes = b"\0asm\x01\0\0\0".to_vec();
    let contents = vec![b'x'; 16_384];
    build::append_metadata_section(&mut bytes, "ic-query:protocol", &contents).unwrap();
    build::append_metadata_section(&mut bytes, "ic-query:build", b"build-evidence").unwrap();
    let facts = inspect(
        &bytes,
        InspectionLimits {
            module_bytes: protocol::WASM_BYTES,
            sections: 10,
            exports: 10,
            custom_sections: 10,
        },
    )
    .unwrap();
    assert_eq!(facts.custom_sections.len(), 2);
    assert_eq!(
        facts.custom_sections[0].name,
        "icp:public ic-query:protocol"
    );
    assert_eq!(facts.custom_sections[0].data, contents);
    assert_eq!(facts.custom_sections[1].name, "icp:public ic-query:build");
    assert_eq!(facts.custom_sections[1].data, b"build-evidence");
    fs::write(&path, &bytes).unwrap();
    assert_eq!(protocol::read_wasm(&path).unwrap(), bytes);
}

#[test]
fn metadata_append_refuses_the_aggregate_module_budget_without_changing_input() {
    let mut bytes = vec![0; protocol::WASM_BYTES - 1];
    let before = Sha256Digest::compute(&bytes);
    let error =
        build::append_metadata_section(&mut bytes, "ic-query:protocol", b"did").unwrap_err();
    assert!(matches!(
        error.downcast_ref::<CustomSectionEncodingError>(),
        Some(CustomSectionEncodingError::LimitExceeded { limit: 1, .. })
    ));
    assert_eq!(Sha256Digest::compute(&bytes), before);
}

#[test]
fn wasm_admission_rejects_malformed_and_oversized_files() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let path = directory.path().join("probe.wasm");
    fs::write(&path, b"not a Wasm module").unwrap();
    let error = protocol::read_wasm(&path).expect_err("malformed artifact rejected");
    assert!(matches!(
        error.downcast_ref::<InspectionError>(),
        Some(InspectionError::Parse(_))
    ));
    fs::File::create(&path)
        .unwrap()
        .set_len(u64::try_from(protocol::WASM_BYTES).unwrap() + 1)
        .unwrap();
    let error = protocol::read_wasm(&path).expect_err("oversized artifact rejected before reading");
    assert!(matches!(
        error.downcast_ref::<ArtifactError>(),
        Some(ArtifactError::LimitExceeded { limit })
            if *limit == u64::try_from(protocol::WASM_BYTES).unwrap()
    ));
}

fn run_fixture(operations: &mut Operations) -> (Result<()>, Value) {
    let mut receipt = fixture_receipt();
    let result = attempt::run_attempt(operations, "local", "probe", &mut receipt);
    (result, serde_json::to_value(receipt).unwrap())
}

#[test]
fn runtime_lifecycle_finishes_only_after_cleanup_and_keeps_failure_identity() {
    let mut operations = Operations::default();
    let (result, receipt) = run_fixture(&mut operations);
    result.unwrap();
    assert_eq!(receipt["status"], "passed");
    assert_eq!(receipt["network_cleanup"], "stopped");
    assert_eq!(
        operations
            .saves
            .iter()
            .find(|save| save["phase"] == "stopping_network")
            .unwrap()["status"],
        "running"
    );
    for cleanup_error in [false, true] {
        let mut operations = Operations {
            startup_error: true,
            cleanup_error,
            ..Default::default()
        };
        let (result, receipt) = run_fixture(&mut operations);
        assert_eq!(result.unwrap_err().to_string(), "startup failed");
        assert_eq!(receipt["failed_phase"], "starting_network");
        assert_eq!(
            receipt["network_cleanup"],
            if cleanup_error { "failed" } else { "stopped" }
        );
        if cleanup_error {
            assert_eq!(receipt["cleanup_error"], "stop failed");
        }
    }
    let mut operations = Operations {
        cleanup_error: true,
        ..Default::default()
    };
    let (result, receipt) = run_fixture(&mut operations);
    assert_eq!(result.unwrap_err().to_string(), "stop failed");
    assert_eq!(receipt["status"], "failed");
}

#[test]
fn runtime_discovery_refusals_do_not_claim_or_stop_an_existing_network() {
    for operations in [
        Operations {
            running: true,
            ..Default::default()
        },
        Operations {
            discovery_error: true,
            ..Default::default()
        },
    ] {
        let mut operations = operations;
        assert!(run_fixture(&mut operations).0.is_err());
        assert_eq!(operations.calls.len(), 1);
        assert!(!operations.verified);
    }
}

#[test]
fn deployed_probe_admission_requires_one_valid_principal() {
    let deployment = json!({"canisters":[
        {"name":"other", "canister_id":"rrkah-fqaaa-aaaaa-aaaaq-cai"},
        {"name":"governance-probe", "canister_id":"aaaaa-aa"},
        {"name":"another", "canister_id":"ryjl3-tyaaa-aaaaa-aaaba-cai"},
    ]});
    assert_eq!(attempt::deployed_canister(&deployment).unwrap(), "aaaaa-aa");
    for deployment in [
        json!({}),
        json!({"canisters":[]}),
        json!({"canisters":[{"name":"other", "canister_id":"aaaaa-aa"}]}),
        json!({"canisters":[{"name":"governance-probe"}]}),
        json!({"canisters":[{"name":"governance-probe", "canister_id":10}]}),
        json!({"canisters":[{"name":"governance-probe", "canister_id":"invalid"}]}),
        json!({"canisters":[{"name":"governance-probe", "canister_id":"aaaaa-aa"}, {"name":"governance-probe", "canister_id":"aaaaa-aa"}]}),
    ] {
        assert!(attempt::deployed_canister(&deployment).is_err());
    }
}

#[test]
fn storage_failure_cannot_skip_cleanup_or_replace_an_operation_error() {
    for phase in ["stopping_network", "finished"] {
        let mut operations = Operations {
            startup_error: true,
            storage_error: Some(phase),
            fail_after_start: true,
            ..Default::default()
        };
        let (result, receipt) = run_fixture(&mut operations);
        assert_eq!(result.unwrap_err().to_string(), "startup failed");
        assert_eq!(receipt["network_cleanup"], "stopped");
        assert!(!receipt["receipt_errors"].as_array().unwrap().is_empty());
        assert_eq!(
            operations.calls.last().unwrap(),
            &["network", "stop", "local"]
        );
    }
    let mut operations = Operations {
        storage_error: Some("stopping_network"),
        ..Default::default()
    };
    let (result, receipt) = run_fixture(&mut operations);
    assert!(result.is_err());
    assert_eq!(receipt["status"], "failed");
    assert_eq!(receipt["network_cleanup"], "stopped");
    let mut operations = Operations {
        cleanup_error: true,
        storage_error: Some("finished"),
        ..Default::default()
    };
    assert_eq!(
        run_fixture(&mut operations).0.unwrap_err().to_string(),
        "stop failed"
    );
}

#[test]
fn receipts_are_atomic_private_and_each_attempt_has_a_fresh_directory() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let path = directory.path().join("receipt.json");
    let mut first = fixture_receipt();
    first.build = Some(
        json!({ "raw_amount":serde_json::from_str::<Value>("184467440737095516160").unwrap(), "padding":"x".repeat(32 * 1024) }),
    );
    first.api_endpoint = Some(json!(42));
    attempt::save_receipt(&path, &first).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&path).unwrap()).unwrap(),
        serde_json::to_value(&first).unwrap()
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let mut passed = fixture_receipt();
    passed.status = receipt::Status::Passed;
    attempt::save_receipt(&path, &passed).unwrap();
    let expected = format!("{}\n", serde_json::to_string_pretty(&passed).unwrap());
    assert_eq!(fs::read(&path).unwrap(), expected.as_bytes());
    let one = receipt_directory(directory.path()).unwrap();
    let two = receipt_directory(directory.path()).unwrap();
    assert_ne!(one, two);
    assert_eq!(
        fs::metadata(one).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let before = fs::read(&path).unwrap();
    let error = write_with(
        &path,
        WriteOptions {
            mode: PublicationMode::Replace,
            permissions: 0o600,
        },
        |file| -> io::Result<()> {
            use io::Write;
            file.write_all(b"partial")?;
            Err(io::ErrorKind::BrokenPipe.into())
        },
    )
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::Producer { source, cleanup_error: None }
            if source.kind() == io::ErrorKind::BrokenPipe)
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    let entries = fs::read_dir(directory.path()).unwrap().count();
    for suffix in ["/", "/.", "//", "/./."] {
        let mut target = path.as_os_str().to_os_string();
        target.push(suffix);
        let error = attempt::save_receipt(Path::new(&target), &fixture_receipt())
            .expect_err("directory-required receipt target is refused");
        assert!(matches!(
            error.downcast_ref::<NamedWriteError<io::Error>>(),
            Some(NamedWriteError::BeforePublication { source, cleanup_error: None })
                if source.kind() == io::ErrorKind::InvalidInput
        ));
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), entries);
    }
}

#[test]
fn command_io_drains_input_and_both_outputs_without_text_loss() {
    let input = vec![b'x'; 512 * 1024];
    let mut child = Command::new("sh");
    child.args(["-c", "cat; printf 'stderr' >&2"]);
    let output = command::supervise(
        &mut child,
        &CommandOptions {
            input: Some(&input),
            stderr: StderrMode::Capture,
            ..Default::default()
        },
        || false,
    )
    .unwrap();
    assert_eq!(output.as_bytes(), input);
    let output = command::supervise(
        &mut build::command(
            Path::new("/tmp"),
            "sh",
            &["-c", "printf 'one\r\ntwo\rthree\n'"],
        ),
        &CommandOptions::default(),
        || false,
    )
    .unwrap();
    assert_eq!(output, "one\ntwo\nthree");
}

#[test]
fn command_exit_failure_preserves_status_and_both_streams_before_text_admission() {
    let mut child = Command::new("sh");
    child.args(["-c", "printf 'stdout'; printf '\\377' >&2; exit 7"]);
    let error = command::supervise(
        &mut child,
        &CommandOptions {
            stderr: StderrMode::Capture,
            handoff: true,
            ..Default::default()
        },
        || false,
    )
    .unwrap_err();
    let error = error.downcast_ref::<CommandFailure>().unwrap();
    let execution = error
        .source
        .downcast_ref::<ic_host_process::tool::ExecutionError>()
        .unwrap();
    assert!(matches!(
        execution.failure,
        ic_host_process::tool::ExecutionFailure::ExitStatus
    ));
    assert_eq!(execution.evidence.status.unwrap().code(), Some(7));
    assert_eq!(error.stdout.as_deref(), Some(b"stdout".as_slice()));
    assert_eq!(error.stderr.as_deref(), Some([255].as_slice()));
    assert!(error.exited);
    assert!(error.cleanup_errors.is_empty());
}

#[test]
fn command_deadline_retains_bytes_and_is_not_replaced_by_text_admission() {
    let mut child = Command::new("sh");
    child.args(["-c", "printf '\\377'; printf 'stderr' >&2; sleep 30"]);
    let started = Instant::now();
    let error = command::supervise(
        &mut child,
        &CommandOptions {
            timeout: Duration::from_millis(100),
            stderr: StderrMode::Capture,
            input: Some(&vec![0; 1024 * 1024]),
            ..Default::default()
        },
        || false,
    )
    .unwrap_err();
    let error = error.downcast_ref::<CommandFailure>().unwrap();
    assert!(error.source.to_string().contains("deadline"));
    assert_eq!(error.stdout.as_deref(), Some([255].as_slice()));
    assert_eq!(error.stderr.as_deref(), Some(b"stderr".as_slice()));
    assert!(started.elapsed() < Duration::from_secs(12));
    assert!(!error.exited);
}

#[test]
fn cancelled_commands_cleanup_and_preserve_their_cancellation() {
    let mut child = Command::new("sh");
    child.args(["-c", "sleep 30"]);
    let error = command::supervise(&mut child, &CommandOptions::default(), || true).unwrap_err();
    assert!(
        error
            .downcast_ref::<CommandFailure>()
            .unwrap()
            .source
            .to_string()
            .contains("cancelled")
    );
}

#[test]
fn background_handoff_requires_successful_text_admission() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let pid = directory.path().join("pid");
    let mut child = Command::new("sh");
    child
        .args([
            "-c",
            "sleep 30 >/dev/null 2>&1 & echo $! > \"$1\"; printf '\\377'",
            "fixture",
        ])
        .arg(&pid);
    let error = command::supervise(
        &mut child,
        &CommandOptions {
            handoff: true,
            ..Default::default()
        },
        || false,
    )
    .unwrap_err();
    assert!(
        error
            .downcast_ref::<CommandFailure>()
            .unwrap()
            .source
            .downcast_ref::<std::str::Utf8Error>()
            .is_some()
    );
    let pid: i32 = fs::read_to_string(pid).unwrap().trim().parse().unwrap();
    // A killed orphan can remain a zombie briefly; a live process must not survive.
    #[cfg(target_os = "linux")]
    if let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) {
        assert!(
            stat.split(')')
                .nth(1)
                .unwrap()
                .trim_start()
                .starts_with('Z')
        );
    }
}

#[test]
fn successful_background_start_survives_handoff_until_explicit_stop() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let pid_file = directory.path().join("pid");
    let mut child = Command::new("sh");
    child
        .args([
            "-c",
            "sleep 30 >/dev/null 2>&1 & echo $! > \"$1\"",
            "fixture",
        ])
        .arg(&pid_file);
    command::supervise(
        &mut child,
        &CommandOptions {
            handoff: true,
            ..Default::default()
        },
        || false,
    )
    .unwrap();
    let pid: i32 = fs::read_to_string(pid_file)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    // This fixture records its own newly started child, which cannot finish naturally yet.
    assert_eq!(unsafe { libc::kill(pid, 0) }, 0);
    assert_eq!(unsafe { libc::kill(pid, libc::SIGKILL) }, 0);
}

#[test]
fn process_fixture() {
    let Ok(mode) = env::var("ICQ_SMOKE_PROCESS_FIXTURE") else {
        return;
    };
    let directory = PathBuf::from(env::var_os("ICQ_SMOKE_FIXTURE_DIRECTORY").unwrap());
    if mode == "restore" {
        // This isolated fixture owns both signals; ignored dispositions make
        // restoration observable without terminating the fixture process.
        for signal in [libc::SIGINT, libc::SIGTERM] {
            assert_ne!(
                unsafe { libc::signal(signal, libc::SIG_IGN) },
                libc::SIG_ERR
            );
        }
        {
            let _signals = command::install_signal_handlers().unwrap();
        }
        for signal in [libc::SIGINT, libc::SIGTERM] {
            assert_eq!(unsafe { libc::raise(signal) }, 0);
        }
        assert_eq!(command::interruption(), None);
        return;
    }
    if mode == "escape" {
        // The child does only async-signal-safe libc calls after fork. It keeps inherited
        // output pipes open outside the owned group until the fixture parent stops it.
        let pid = unsafe { libc::fork() };
        assert!(pid >= 0);
        if pid == 0 {
            unsafe {
                libc::setsid();
                libc::sleep(30);
                libc::_exit(0);
            }
        }
        fs::write(directory.join("escaped-pid"), pid.to_string()).unwrap();
        return;
    }
    assert_eq!(mode, "signal");
    let _signals = command::install_signal_handlers().unwrap();
    struct SignalOperations {
        directory: PathBuf,
    }
    impl AttemptOperations for SignalOperations {
        fn publish(&mut self, receipt: &Receipt) -> Result<()> {
            attempt::save_receipt(&self.directory.join("receipt.json"), receipt)
        }
        fn verify(
            &mut self,
            _environment: &str,
            _canister: &str,
            _receipt: &mut Receipt,
        ) -> Result<()> {
            panic!("interrupted startup cannot verify")
        }
        fn icp(&mut self, args: &[&str], options: &CommandOptions<'_>) -> Result<String> {
            match args {
                ["network", "status", "local"] => Err(exit_failure()),
                ["network", "start", "local", "--background"] => {
                    let mut child = Command::new("sh");
                    child
                        .args(["-c", "echo $$ > \"$1\"; sleep 30", "fixture"])
                        .arg(self.directory.join("active-pid"));
                    command::run(&mut child, options)
                }
                ["network", "stop", "local"] => {
                    assert!(options.cleanup);
                    fs::write(self.directory.join("stopped"), "stopped")?;
                    Ok(String::new())
                }
                _ => panic!("unexpected interruption fixture operation"),
            }
        }
    }
    let mut receipt = fixture_receipt();
    let error = attempt::run_attempt(
        &mut SignalOperations { directory },
        "local",
        "probe",
        &mut receipt,
    )
    .unwrap_err();
    assert!(
        error
            .downcast_ref::<CommandFailure>()
            .unwrap()
            .interrupted
            .is_some()
    );
}

#[test]
fn signal_handler_guard_restores_prior_dispositions() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-signal-restore").unwrap();
    let mut child = Command::new(env::current_exe().unwrap());
    child
        .args(["--exact", "tests::process_fixture", "--nocapture"])
        .env("ICQ_SMOKE_PROCESS_FIXTURE", "restore")
        .env("ICQ_SMOKE_FIXTURE_DIRECTORY", directory.path());
    command::run(
        &mut child,
        &CommandOptions {
            timeout: Duration::from_secs(10),
            ..CommandOptions::default()
        },
    )
    .unwrap();
}

#[test]
fn signals_during_startup_publish_failure_and_still_stop_the_network() {
    use ic_host_process::{
        child::{CleanupPolicy, OwnedChild},
        tool::{OutputLimit, OutputLimits, SuccessfulExit, communicate_child},
    };
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
        let marker = directory.path().join("active-pid");
        let mut command = Command::new(env::current_exe().unwrap());
        command
            .args(["--exact", "tests::process_fixture", "--nocapture"])
            .env("ICQ_SMOKE_PROCESS_FIXTURE", "signal")
            .env("ICQ_SMOKE_FIXTURE_DIRECTORY", directory.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = OwnedChild::spawn(&mut command, CleanupPolicy::KillAndWait).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !marker.exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        let runner_pid = i32::try_from(child.id()).unwrap();
        assert_eq!(unsafe { libc::kill(runner_pid, signal) }, 0);
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(
            unsafe { libc::kill(runner_pid, libc::SIGTERM) },
            0,
            "a second signal must not interrupt cleanup"
        );
        let evidence = communicate_child(
            &mut child,
            None,
            OutputLimits {
                stdout: OutputLimit::Terminate(64 * 1024),
                stderr: OutputLimit::Terminate(64 * 1024),
                timeout: Some(Duration::from_secs(15)),
            },
            SuccessfulExit::Cleanup,
            || false,
        )
        .unwrap();
        assert!(evidence.status.success());
        let receipt: Value =
            serde_json::from_slice(&fs::read(directory.path().join("receipt.json")).unwrap())
                .unwrap();
        assert_eq!(receipt["status"], "failed");
        assert_eq!(receipt["failed_phase"], "starting_network");
        assert_eq!(receipt["interrupted_by"], command::signal_name(signal));
        assert_eq!(receipt["network_cleanup"], "stopped");
        assert!(directory.path().join("stopped").exists());
    }
}

#[test]
fn escaped_descendant_output_pipes_do_not_extend_the_deadline() {
    let directory = FixtureDirectory::new(&env::temp_dir(), "ic-query-smoke").unwrap();
    let pid_file = directory.path().join("escaped-pid");
    let mut child = Command::new(env::current_exe().unwrap());
    child
        .args(["--exact", "tests::process_fixture", "--nocapture"])
        .env("ICQ_SMOKE_PROCESS_FIXTURE", "escape")
        .env("ICQ_SMOKE_FIXTURE_DIRECTORY", directory.path());
    let started = Instant::now();
    let error = command::supervise(
        &mut child,
        &CommandOptions {
            timeout: Duration::from_secs(1),
            ..Default::default()
        },
        || false,
    )
    .unwrap_err();
    let pid: i32 = fs::read_to_string(pid_file).unwrap().parse().unwrap();
    // The explicit fixture owns cleanup for this deliberately escaped descendant.
    assert_eq!(unsafe { libc::kill(pid, libc::SIGKILL) }, 0);
    assert!(started.elapsed() < Duration::from_secs(12));
    assert!(
        error
            .downcast_ref::<CommandFailure>()
            .unwrap()
            .source
            .to_string()
            .contains("deadline")
    );
}

fn fixture_receipt() -> Receipt {
    Receipt::new(
        "local",
        "fixture icp".to_string(),
        "fixture rustc".to_string(),
        "2026-10-29T00:00:00Z".to_string(),
    )
}
