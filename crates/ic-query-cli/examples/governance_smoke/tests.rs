use super::*;
use command::{CommandFailure, CommandOptions, StderrMode};
use ic_host_artifacts::{
    artifact::{ArtifactError, Sha256Digest},
    wasm::InspectionError,
};
use ic_host_fs::durable::{NamedWriteError, PublicationMode, WriteOptions, write_with};
use protocol::Probe;
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    time::{Duration, Instant},
};

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        let root = env::temp_dir().join(format!(
            "ic-query-smoke-{}-{}",
            std::process::id(),
            time::OffsetDateTime::now_utc().unix_timestamp_nanos()
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        Self(root)
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
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
fn actions_are_admitted_before_effects_and_help_commands_are_sorted() {
    assert_eq!(parse(&[]).unwrap(), Action::Help);
    assert_eq!(parse(&["help".into()]).unwrap(), Action::Help);
    assert_eq!(
        parse(&[
            "verify-mainnet".into(),
            "--canister".into(),
            "aaaaa-aa".into()
        ])
        .unwrap(),
        Action::VerifyMainnet("aaaaa-aa".into())
    );
    for args in [
        vec!["verify-mainnet"],
        vec!["local", "--canister", "aaaaa-aa"],
        vec!["build", "extra"],
    ] {
        assert!(parse(&args.into_iter().map(OsString::from).collect::<Vec<_>>()).is_err());
    }
    let commands: Vec<_> = USAGE.lines().skip(3).collect();
    let mut sorted = commands.clone();
    sorted.sort_unstable();
    assert_eq!(commands, sorted);
}

#[test]
fn source_fingerprint_preserves_sorted_paths_raw_contents_and_complete_inputs() {
    let directory = Directory::new();
    for path in [
        "tests/canister/probe.did",
        "rust-toolchain.toml",
        "crates/ic-query/src/lib.rs",
        "crates/ic-query/Cargo.toml",
        "crates/ic-query-cli/examples/governance_smoke/main.rs",
        "crates/ic-query-cli/Cargo.toml",
        "Cargo.toml",
        "Cargo.lock",
    ] {
        let path = directory.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"fixture\n").unwrap();
    }
    let source = directory.0.join("crates/ic-query/src/large.rs");
    fs::write(&source, vec![b'x'; 65_537]).unwrap();
    let expected = "2c947bca9ebcb90408ef8ba4a6f62b3a3992c82f23d18b684ff34ee4162c405b";
    assert_eq!(build::source_digest(&directory.0).unwrap(), expected);
    fs::write(directory.0.join("crates/ic-query/README.md"), b"guide").unwrap();
    assert_eq!(build::source_digest(&directory.0).unwrap(), expected);
    fs::rename(&source, source.with_file_name("renamed.rs")).unwrap();
    assert_ne!(build::source_digest(&directory.0).unwrap(), expected);
    fs::remove_file(directory.0.join("Cargo.lock")).unwrap();
    assert_eq!(
        build::source_digest(&directory.0)
            .unwrap_err()
            .downcast_ref::<io::Error>()
            .unwrap()
            .kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn persistent_identity_selection_resolves_missing_paths_and_symlinks() {
    let directory = Directory::new();
    let root = &directory.0;
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
    assert_eq!(command.get_current_dir(), Some(root.as_path()));
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
fn candid_admission_preserves_large_text_and_raw_numbers() {
    let text = format!(
        "{{\"raw_amount\":184467440737095516160,\"padding\":\"{}\"}}",
        "x".repeat(1024 * 1024)
    );
    let decoded = protocol::decode_reply(&candid::encode_one(&text).unwrap()).unwrap();
    assert_eq!(decoded, text);
    let json: Value = serde_json::from_str(&decoded).unwrap();
    assert_eq!(json["raw_amount"].to_string(), "184467440737095516160");
}

#[test]
fn candid_admission_rejects_wrong_types_extra_values_trailing_bytes_and_large_headers() {
    let mut trailing = candid::encode_one("{}").unwrap();
    trailing.push(0);
    let mut truncated = candid::encode_one("payload").unwrap();
    truncated.pop();
    // A record header large enough to exceed admission even though the reply is small.
    fn leb(mut number: u32, bytes: &mut Vec<u8>) {
        while number >= 128 {
            bytes.push((number as u8 & 127) | 128);
            number >>= 7;
        }
        bytes.push(number as u8);
    }
    let mut header = b"DIDL\x01\x6c".to_vec();
    leb(30_000, &mut header);
    for field in 0..30_000 {
        leb(field, &mut header);
        header.push(0x71);
    }
    header.extend([1, 0]);
    for bytes in [
        candid::encode_one(7_u64).unwrap(),
        candid::encode_args(("{}", "{}")).unwrap(),
        candid::encode_args(("{}", vec![(); 100_000])).unwrap(),
        trailing,
        truncated,
        b"DIDL\x81\x20".to_vec(),
        header,
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
    let directory = Directory::new();
    for (name, bytes) in [
        ("invalid", b"not Candid".to_vec()),
        ("oversized", vec![0; 2 * 1024 * 1024 + 1]),
    ] {
        let path = directory.0.join(name);
        assert!(protocol::publish_reply(&path, &bytes).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
        let error = protocol::publish_reply(&path, &candid::encode_one("{}").unwrap()).unwrap_err();
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
    for (key, value) in [
        ("schema_version", json!(2)),
        ("network", json!("local")),
        ("governance_canister_id", json!("aaaaa-aa")),
        ("source", json!({"source_transport":"replica_query"})),
        ("fetched_at", json!("2026-02-30T12:00:00Z")),
        ("fetched_at", json!("2026-09-15T12:00:00+00:00")),
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
        Ok(if name == "candid:service" {
            b"service : {};\n".to_vec()
        } else {
            b"{\"schema_version\":1}".to_vec()
        })
    }
    fn report(&mut self, kind: &str) -> Result<Vec<u8>> {
        if self.invalid_kind == Some(kind) {
            return Ok(b"invalid Candid".to_vec());
        }
        Ok(candid::encode_one(serde_json::to_string(&payload(kind))?)?)
    }
}

fn verify_fixture(
    directory: &Directory,
    changed: bool,
    invalid_kind: Option<&'static str>,
    bad_wasm: bool,
) -> (Result<()>, Value) {
    let bytes = if bad_wasm {
        b"not Wasm".as_slice()
    } else {
        b"\0asm\x01\0\0\0".as_slice()
    };
    let wasm = directory.0.join("probe.wasm");
    fs::write(&wasm, bytes).unwrap();
    let mut probe = FixtureProbe {
        hash: Sha256Digest::compute(bytes).to_string(),
        changed,
        calls: 0,
        invalid_kind,
    };
    let mut receipt = json!({"status":"running", "environment":"local"});
    let output = directory.0.join("receipt.json");
    let result = attempt::verify(
        &mut probe,
        "aaaaa-aa",
        &wasm,
        b"service : {};\n",
        &output,
        &mut receipt,
        |receipt| attempt::save_receipt(&output, receipt),
    );
    (result, receipt)
}

#[test]
fn verification_preserves_reports_raw_replies_and_final_module_check() {
    let directory = Directory::new();
    let (result, receipt) = verify_fixture(&directory, false, None, false);
    result.unwrap();
    assert_eq!(receipt["reports"].as_object().unwrap().len(), 4);
    assert_eq!(receipt["module_hash_before"], receipt["module_hash_after"]);
    for kind in KINDS {
        let bytes = fs::read(directory.0.join(format!("{kind}.candid"))).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&protocol::decode_reply(&bytes).unwrap()).unwrap(),
            payload(kind)
        );
    }
    let directory = Directory::new();
    let (result, receipt) = verify_fixture(&directory, true, None, false);
    assert!(result.is_err());
    assert_eq!(receipt["reports"].as_object().unwrap().len(), 4);
}

#[test]
fn invalid_reply_retains_prior_reports_and_a_checkpoint_before_admission() {
    let directory = Directory::new();
    let (result, receipt) = verify_fixture(&directory, false, Some("metrics"), false);
    assert!(result.is_err());
    assert!(receipt["reports"]["economics"].get("report").is_some());
    assert_eq!(
        receipt["reports"]["metrics"],
        json!({"reply_file":"metrics.candid"})
    );
    assert_eq!(
        fs::read(directory.0.join("metrics.candid")).unwrap(),
        b"invalid Candid"
    );
    let persisted: Value =
        serde_json::from_slice(&fs::read(directory.0.join("receipt.json")).unwrap()).unwrap();
    assert_eq!(persisted["phase"], "collecting_metrics");
    let directory = Directory::new();
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
                        cleanup_errors: vec!["SIGKILL: refused".into()],
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
    fn publish(&mut self, receipt: &Value) -> Result<()> {
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
    fn verify(&mut self, _environment: &str, canister: &str, receipt: &mut Value) -> Result<()> {
        assert_eq!(canister, "aaaaa-aa");
        assert_eq!(receipt["status"], "running");
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
    assert_eq!(
        error
            .downcast_ref::<CommandFailure>()
            .unwrap()
            .source
            .to_string(),
        "startup deadline"
    );
    assert_eq!(receipt["error"], "startup deadline");
    assert_eq!(receipt["command_stdout"], "\u{fffd}");
    assert_eq!(receipt["command_stderr"], "command error");
    assert_eq!(
        receipt["command_cleanup_errors"],
        json!(["SIGKILL: refused"])
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
    let directory = Directory::new();
    let path = directory.0.join("probe.wasm");
    let mut bytes = b"\0asm\x01\0\0\0".to_vec();
    let contents = vec![b'x'; 16_384];
    let section = build::metadata_section("candid:service", &contents);
    assert!(section.ends_with(&contents));
    bytes.extend(section);
    fs::write(&path, &bytes).unwrap();
    assert_eq!(protocol::read_wasm(&path).unwrap(), bytes);
}

#[test]
fn wasm_admission_rejects_malformed_and_oversized_files() {
    let directory = Directory::new();
    let path = directory.0.join("probe.wasm");
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
    let mut receipt = json!({"schema_version":1, "status":"running", "environment":"local"});
    let result = attempt::run_attempt(operations, "local", "probe", &mut receipt);
    (result, receipt)
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
    let directory = Directory::new();
    let path = directory.0.join("receipt.json");
    let first = json!({
        "schema_version":1,
        "status":"running",
        "raw_amount":serde_json::from_str::<Value>("184467440737095516160").unwrap(),
        "padding":"x".repeat(32 * 1024),
    });
    attempt::save_receipt(&path, &first).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&path).unwrap()).unwrap(),
        first
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    attempt::save_receipt(&path, &json!({"status":"passed"})).unwrap();
    assert_eq!(
        fs::read(&path).unwrap(),
        b"{\n  \"status\": \"passed\"\n}\n"
    );
    let one = receipt_directory(&directory.0).unwrap();
    let two = receipt_directory(&directory.0).unwrap();
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
    assert_eq!(fs::read(path).unwrap(), before);
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
    let directory = Directory::new();
    let pid = directory.0.join("pid");
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
    let directory = Directory::new();
    let pid_file = directory.0.join("pid");
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
    let _signals = command::SignalGuard::install().unwrap();
    struct SignalOperations {
        directory: PathBuf,
    }
    impl AttemptOperations for SignalOperations {
        fn publish(&mut self, receipt: &Value) -> Result<()> {
            attempt::save_receipt(&self.directory.join("receipt.json"), receipt)
        }
        fn verify(
            &mut self,
            _environment: &str,
            _canister: &str,
            _receipt: &mut Value,
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
    let mut receipt = json!({"schema_version":1,"status":"running","environment":"local"});
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
fn signals_during_startup_publish_failure_and_still_stop_the_network() {
    use ic_host_process::{
        child::OwnedChild,
        tool::{OutputLimits, SuccessfulExit, communicate_child},
    };
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let directory = Directory::new();
        let marker = directory.0.join("active-pid");
        let mut command = Command::new(env::current_exe().unwrap());
        command
            .args(["--exact", "tests::process_fixture", "--nocapture"])
            .env("ICQ_SMOKE_PROCESS_FIXTURE", "signal")
            .env("ICQ_SMOKE_FIXTURE_DIRECTORY", &directory.0)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = OwnedChild::spawn(&mut command).unwrap();
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
                stdout_bytes: 64 * 1024,
                stderr_bytes: 64 * 1024,
                timeout: Duration::from_secs(15),
            },
            SuccessfulExit::Cleanup,
            || false,
        )
        .unwrap();
        assert!(evidence.status.unwrap().success());
        let receipt: Value =
            serde_json::from_slice(&fs::read(directory.0.join("receipt.json")).unwrap()).unwrap();
        assert_eq!(receipt["status"], "failed");
        assert_eq!(receipt["failed_phase"], "starting_network");
        assert_eq!(receipt["interrupted_by"], command::signal_name(signal));
        assert_eq!(receipt["network_cleanup"], "stopped");
        assert!(directory.0.join("stopped").exists());
    }
}

#[test]
fn escaped_descendant_output_pipes_do_not_extend_the_deadline() {
    let directory = Directory::new();
    let pid_file = directory.0.join("escaped-pid");
    let mut child = Command::new(env::current_exe().unwrap());
    child
        .args(["--exact", "tests::process_fixture", "--nocapture"])
        .env("ICQ_SMOKE_PROCESS_FIXTURE", "escape")
        .env("ICQ_SMOKE_FIXTURE_DIRECTORY", &directory.0);
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
