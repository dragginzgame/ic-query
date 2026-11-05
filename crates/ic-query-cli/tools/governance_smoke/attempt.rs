//! Verification admission, durable checkpoints and owned runtime cleanup.

use crate::{
    GOVERNANCE, Result,
    command::{self, CommandFailure, CommandOptions, StderrMode},
    invalid,
    protocol::{self, Probe},
    receipt::{NetworkCleanup, Phase, Receipt, ReportKind, ReportReceipt, Status},
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::durable::{PublicationMode, WriteOptions, write_with};
use ic_host_text::timestamp::parse_utc_timestamp_secs;
use serde_json::{Value, json};
use std::{
    io::{self, BufWriter, Write},
    path::Path,
    time::Duration,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub fn utc_now() -> Result<String> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}

pub fn save_receipt(path: &Path, receipt: &Receipt) -> Result<()> {
    write_with(
        path,
        WriteOptions {
            mode: PublicationMode::Replace,
            permissions: 0o600,
        },
        |file| -> io::Result<()> {
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, receipt)?;
            writer.write_all(b"\n")?;
            writer.flush()
        },
    )?;
    Ok(())
}

pub fn validate_report(kind: &str, mut payload: Value, collector: &str) -> Result<Value> {
    if payload["status"] != "ok" {
        return Err(invalid(&format!(
            "{kind}: {}",
            payload.get("error").map_or_else(
                || "missing successful response".to_string(),
                ToString::to_string
            )
        )));
    }
    let report = &payload["report"];
    if report["schema_version"] != 1
        || report["network"] != "ic"
        || report["governance_canister_id"] != GOVERNANCE
        || report["source"]
            != json!({"source_transport":"replicated_inter_canister_call", "collector_canister_id":collector})
        || report.get(kind).is_none()
    {
        return Err(invalid(&format!(
            "{kind}: incorrect report identity or replicated provenance"
        )));
    }
    let timestamp = report["fetched_at"]
        .as_str()
        .ok_or_else(|| invalid("missing fetched_at"))?;
    if timestamp.len() != 20 || parse_utc_timestamp_secs(timestamp).is_none() {
        return Err(invalid(
            "fetched_at must be canonical second-resolution UTC",
        ));
    }
    Ok(payload["report"].take())
}

pub fn deployed_canister(deployment: &Value) -> Result<String> {
    let items = deployment["canisters"]
        .as_array()
        .ok_or_else(|| invalid("deployment omitted canisters"))?;
    let mut selected = items
        .iter()
        .filter(|item| item["name"] == "governance-probe");
    let (Some(item), None) = (selected.next(), selected.next()) else {
        return Err(invalid(
            "deployment must identify exactly one Governance probe",
        ));
    };
    let principal = item["canister_id"]
        .as_str()
        .ok_or_else(|| invalid("deployment omitted probe principal"))?;
    ic_platform::candid::contract::Principal::from_text(principal)?;
    Ok(principal.to_string())
}

pub fn checkpoint(
    receipt: &mut Receipt,
    phase: Option<Phase>,
    publish: &mut impl FnMut(&Receipt) -> Result<()>,
) -> Result<()> {
    if receipt.status == Status::Running
        && !matches!(phase, Some(Phase::StoppingNetwork | Phase::Finished))
    {
        command::check_interruption()?;
    }
    if let Some(phase) = phase {
        receipt.phase = Some(phase);
    }
    receipt.updated_at = Some(utc_now()?);
    publish(receipt)
}

pub fn verify(
    probe: &mut impl Probe,
    canister: &str,
    wasm: &Path,
    protocol: &[u8],
    output: &Path,
    receipt: &mut Receipt,
    mut publish: impl FnMut(&Receipt) -> Result<()>,
) -> Result<()> {
    receipt.canister_id = Some(canister.to_string());
    checkpoint(receipt, Some(Phase::CheckingModule), &mut publish)?;
    let before = probe.module_hash()?;
    receipt.module_hash_before = Some(before.clone());
    checkpoint(receipt, None, &mut publish)?;
    if before != Sha256Digest::compute(&protocol::read_wasm(wasm)?).to_string() {
        return Err(invalid(
            "deployed module hash differs from the locally built probe",
        ));
    }
    checkpoint(receipt, Some(Phase::CheckingProtocol), &mut publish)?;
    let metadata = probe.metadata("ic-query:protocol")?;
    if metadata != protocol {
        return Err(invalid(
            "deployed Wire metadata differs from probe-wire.json",
        ));
    }
    receipt.protocol_sha256 = Some(Sha256Digest::compute(&metadata).to_string());
    checkpoint(receipt, Some(Phase::CheckingBuildMetadata), &mut publish)?;
    receipt.build = Some(serde_json::from_slice(&probe.metadata("ic-query:build")?)?);
    receipt.reports = Some(std::collections::BTreeMap::new());
    checkpoint(receipt, None, &mut publish)?;
    for report_kind in ReportKind::ALL {
        let kind = report_kind.as_str();
        println!("Collecting {kind} ({})", receipt.environment);
        checkpoint(receipt, Some(Phase::Collecting(report_kind)), &mut publish)?;
        let reply = format!("{kind}.wire");
        receipt
            .reports
            .as_mut()
            .expect("report collection started")
            .insert(
                report_kind,
                ReportReceipt {
                    reply_file: reply.clone(),
                    report: None,
                },
            );
        checkpoint(receipt, None, &mut publish)?;
        let response =
            protocol::publish_reply(&output.with_file_name(&reply), &probe.report(kind)?)?;
        checkpoint(receipt, Some(Phase::Validating(report_kind)), &mut publish)?;
        let payload = serde_json::from_str(&response)?;
        receipt
            .reports
            .as_mut()
            .expect("report collection started")
            .get_mut(&report_kind)
            .expect("reply checkpoint exists")
            .report = Some(validate_report(kind, payload, canister)?);
        checkpoint(receipt, Some(Phase::Collected(report_kind)), &mut publish)?;
    }
    checkpoint(receipt, Some(Phase::CheckingFinalModule), &mut publish)?;
    let after = probe.module_hash()?;
    receipt.module_hash_after = Some(after.clone());
    checkpoint(receipt, None, &mut publish)?;
    if before != after {
        return Err(invalid("probe module changed during collection"));
    }
    Ok(())
}

///
/// AttemptOperations
///
/// Runtime, verification and receipt IO replaced by fixtures in lifecycle tests.
///

pub trait AttemptOperations {
    fn icp(&mut self, args: &[&str], options: &CommandOptions<'_>) -> Result<String>;
    fn publish(&mut self, receipt: &Receipt) -> Result<()>;
    fn verify(&mut self, environment: &str, canister: &str, receipt: &mut Receipt) -> Result<()>;
}

fn attempt_checkpoint(
    operations: &mut impl AttemptOperations,
    receipt: &mut Receipt,
    phase: Option<Phase>,
) -> Result<()> {
    checkpoint(receipt, phase, &mut |receipt| operations.publish(receipt))
}

fn retain_failure(receipt: &mut Receipt, error: &(dyn std::error::Error + 'static)) {
    receipt.status = Status::Failed;
    if receipt.failed_phase.is_none() {
        receipt.failed_phase = receipt.phase;
    }
    receipt.error = Some(error.to_string());
    let interrupted = error
        .downcast_ref::<CommandFailure>()
        .and_then(|error| error.interrupted)
        .or_else(command::interruption);
    if let Some(signal) = interrupted {
        receipt.interrupted_by = Some(command::signal_name(signal).to_string());
    }
    if let Some(error) = error.downcast_ref::<CommandFailure>() {
        for (target, bytes) in [
            (&mut receipt.command_stdout, &error.stdout),
            (&mut receipt.command_stderr, &error.stderr),
        ] {
            if let Some(bytes) = bytes {
                *target = Some(String::from_utf8_lossy(bytes).into_owned());
            }
        }
        if !error.cleanup_errors.is_empty() {
            receipt.command_cleanup_errors = Some(
                error
                    .cleanup_errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            );
        }
    }
}

fn retain_storage_failure(receipt: &mut Receipt, error: &dyn std::error::Error) {
    receipt
        .storage_errors
        .get_or_insert_with(Vec::new)
        .push(error.to_string());
    eprintln!("Receipt publication failed: {error}");
}

pub fn run_attempt(
    operations: &mut impl AttemptOperations,
    environment: &str,
    canister: &str,
    receipt: &mut Receipt,
) -> Result<()> {
    // No runtime is owned until startup is dispatched; an initial IO refusal ends here.
    attempt_checkpoint(operations, receipt, Some(Phase::Starting))?;
    let mut startup_attempted = false;
    let result = collect_attempt(
        operations,
        environment,
        canister,
        receipt,
        &mut startup_attempted,
    );
    let verified = result.is_ok();
    let mut primary = result.err();
    if let Some(error) = &primary {
        retain_failure(receipt, error.as_ref());
        if let Err(storage) = attempt_checkpoint(operations, receipt, None) {
            retain_storage_failure(receipt, storage.as_ref());
        }
    }
    if startup_attempted {
        receipt.network_cleanup = Some(NetworkCleanup::Pending);
        let checkpoint_error =
            attempt_checkpoint(operations, receipt, Some(Phase::StoppingNetwork)).err();
        if let Some(error) = &checkpoint_error {
            retain_storage_failure(receipt, error.as_ref());
        }
        // Even interruption or receipt failure cannot skip the owned network's stop.
        let stop = operations.icp(
            &["network", "stop", "local"],
            &CommandOptions {
                capture: false,
                timeout: Duration::from_secs(30),
                cleanup: true,
                ..Default::default()
            },
        );
        receipt.network_cleanup = Some(if stop.is_ok() {
            NetworkCleanup::Stopped
        } else {
            NetworkCleanup::Failed
        });
        if let Some(error) = stop.err().or(checkpoint_error) {
            receipt.status = Status::Failed;
            if receipt.failed_phase.is_none() {
                receipt.failed_phase = receipt.phase;
            }
            receipt.cleanup_error = Some(error.to_string());
            if primary.is_none() {
                primary = Some(error);
            }
        }
    }
    if primary.is_none()
        && let Err(error) = command::check_interruption()
    {
        retain_failure(receipt, error.as_ref());
        primary = Some(error);
    }
    if verified && receipt.status == Status::Running {
        receipt.status = Status::Passed;
    }
    receipt.finished_at = Some(utc_now()?);
    if let Err(storage) = attempt_checkpoint(operations, receipt, Some(Phase::Finished)) {
        if primary.is_none() {
            primary = Some(storage);
        } else {
            retain_storage_failure(receipt, storage.as_ref());
        }
    }
    primary.map_or(Ok(()), Err)
}

fn collect_attempt(
    operations: &mut impl AttemptOperations,
    environment: &str,
    canister: &str,
    receipt: &mut Receipt,
    startup_attempted: &mut bool,
) -> Result<()> {
    let canister = if environment == "local" {
        attempt_checkpoint(operations, receipt, Some(Phase::CheckingNetwork))?;
        match operations.icp(
            &["network", "status", "local"],
            &CommandOptions {
                timeout: Duration::from_secs(30),
                stderr: StderrMode::Discard,
                ..Default::default()
            },
        ) {
            Ok(_) => {
                return Err(invalid(
                    "the harness network is already running; stop it before running the smoke",
                ));
            }
            Err(error)
                if error.downcast_ref::<CommandFailure>().is_some_and(|error| {
                    error.exited && error.cleanup_errors.is_empty() && error.interrupted.is_none()
                }) => {}
            Err(error) => return Err(error),
        }
        attempt_checkpoint(operations, receipt, Some(Phase::StartingNetwork))?;
        *startup_attempted = true;
        operations.icp(
            &["network", "start", "local", "--background"],
            &CommandOptions {
                capture: false,
                timeout: Duration::from_mins(15),
                handoff: true,
                ..Default::default()
            },
        )?;
        attempt_checkpoint(operations, receipt, Some(Phase::ReadingNetwork))?;
        let network: Value = serde_json::from_str(&operations.icp(
            &["network", "status", "local", "--json"],
            &CommandOptions::default(),
        )?)?;
        receipt.api_endpoint = Some(network["api_url"].clone());
        receipt.network = Some(network);
        if receipt
            .api_endpoint
            .as_ref()
            .and_then(Value::as_str)
            .is_none()
        {
            return Err(invalid("network discovery omitted api_url"));
        }
        attempt_checkpoint(operations, receipt, Some(Phase::Deploying))?;
        let deployment = serde_json::from_str(&operations.icp(
            &["deploy", "-e", "local", "--json"],
            &CommandOptions::default(),
        )?)?;
        deployed_canister(&deployment)?
    } else {
        receipt.api_endpoint = Some("https://icp-api.io/".into());
        attempt_checkpoint(operations, receipt, Some(Phase::Building))?;
        operations.icp(
            &["build", "-e", environment],
            &CommandOptions {
                capture: false,
                ..Default::default()
            },
        )?;
        canister.to_string()
    };
    operations.verify(environment, &canister, receipt)
}
