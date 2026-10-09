//! Verification admission, durable checkpoints and owned runtime cleanup.

use crate::{
    GOVERNANCE, KINDS, Result,
    command::{self, CommandFailure, CommandOptions, StderrMode},
    invalid,
    protocol::{self, Probe},
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::durable::{PublicationMode, WriteOptions, write_typed_with};
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    path::Path,
    time::Duration,
};
use time::{
    OffsetDateTime, PrimitiveDateTime,
    format_description::{self, well_known::Rfc3339},
};

pub fn utc_now() -> Result<String> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}

pub fn save_receipt(path: &Path, receipt: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(receipt)?;
    bytes.push(b'\n');
    write_typed_with(
        path,
        WriteOptions {
            mode: PublicationMode::Replace,
            permissions: 0o600,
        },
        |file| -> io::Result<()> { file.write_all(&bytes) },
    )?;
    Ok(())
}

pub fn validate_report(kind: &str, payload: &Value, collector: &str) -> Result<Value> {
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
    let format =
        format_description::parse_borrowed::<2>("[year]-[month]-[day]T[hour]:[minute]:[second]Z")?;
    let parsed = PrimitiveDateTime::parse(timestamp, &format)?;
    if timestamp.len() != 20 || parsed.format(&format)? != timestamp {
        return Err(invalid(
            "fetched_at must be canonical second-resolution UTC",
        ));
    }
    Ok(report.clone())
}

pub fn deployed_canister(deployment: &Value) -> Result<String> {
    let items = deployment["canisters"]
        .as_array()
        .ok_or_else(|| invalid("deployment omitted canisters"))?;
    let selected: Vec<_> = items
        .iter()
        .filter(|item| item["name"] == "governance-probe")
        .collect();
    let [item] = selected.as_slice() else {
        return Err(invalid(
            "deployment must identify exactly one Governance probe",
        ));
    };
    let principal = item["canister_id"]
        .as_str()
        .ok_or_else(|| invalid("deployment omitted probe principal"))?;
    candid::Principal::from_text(principal)?;
    Ok(principal.to_string())
}

pub fn checkpoint(
    receipt: &mut Value,
    phase: Option<&str>,
    publish: &mut impl FnMut(&Value) -> Result<()>,
) -> Result<()> {
    if receipt["status"] == "running" && !matches!(phase, Some("stopping_network" | "finished")) {
        command::check_interruption()?;
    }
    if let Some(phase) = phase {
        receipt["phase"] = phase.into();
    }
    receipt["updated_at"] = utc_now()?.into();
    publish(receipt)
}

pub fn verify(
    probe: &mut impl Probe,
    canister: &str,
    wasm: &Path,
    candid: &[u8],
    output: &Path,
    receipt: &mut Value,
    mut publish: impl FnMut(&Value) -> Result<()>,
) -> Result<()> {
    receipt["canister_id"] = canister.into();
    checkpoint(receipt, Some("checking_module"), &mut publish)?;
    let before = probe.module_hash()?;
    receipt["module_hash_before"] = before.clone().into();
    checkpoint(receipt, None, &mut publish)?;
    if before != protocol::inspect_wasm(wasm)?.to_string() {
        return Err(invalid(
            "deployed module hash differs from the locally built probe",
        ));
    }
    checkpoint(receipt, Some("checking_candid"), &mut publish)?;
    let metadata = probe.metadata("candid:service")?;
    if metadata != candid {
        return Err(invalid("deployed Candid metadata differs from probe.did"));
    }
    receipt["candid_sha256"] = Sha256Digest::compute(&metadata).to_string().into();
    checkpoint(receipt, Some("checking_build_metadata"), &mut publish)?;
    receipt["build"] = serde_json::from_slice(&probe.metadata("ic-query:build")?)?;
    receipt["reports"] = json!({});
    checkpoint(receipt, None, &mut publish)?;
    for kind in KINDS {
        println!(
            "Collecting {kind} ({})",
            receipt["environment"].as_str().unwrap_or_default()
        );
        checkpoint(receipt, Some(&format!("collecting_{kind}")), &mut publish)?;
        let reply = format!("{kind}.candid");
        receipt["reports"][kind] = json!({"reply_file":reply});
        checkpoint(receipt, None, &mut publish)?;
        let response =
            protocol::publish_reply(&output.with_file_name(&reply), &probe.report(kind)?)?;
        checkpoint(receipt, Some(&format!("validating_{kind}")), &mut publish)?;
        let payload = serde_json::from_str(&response)?;
        receipt["reports"][kind]["report"] = validate_report(kind, &payload, canister)?;
        checkpoint(receipt, Some(&format!("collected_{kind}")), &mut publish)?;
    }
    checkpoint(receipt, Some("checking_final_module"), &mut publish)?;
    let after = probe.module_hash()?;
    receipt["module_hash_after"] = after.clone().into();
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
    fn publish(&mut self, receipt: &Value) -> Result<()>;
    fn verify(&mut self, environment: &str, canister: &str, receipt: &mut Value) -> Result<()>;
}

fn attempt_checkpoint(
    operations: &mut impl AttemptOperations,
    receipt: &mut Value,
    phase: Option<&str>,
) -> Result<()> {
    checkpoint(receipt, phase, &mut |receipt| operations.publish(receipt))
}

fn retain_failure(receipt: &mut Value, error: &(dyn std::error::Error + 'static)) {
    receipt["status"] = "failed".into();
    if receipt.get("failed_phase").is_none() {
        receipt["failed_phase"] = receipt["phase"].clone();
    }
    receipt["error"] = error.to_string().into();
    let interrupted = error
        .downcast_ref::<CommandFailure>()
        .and_then(|error| error.interrupted)
        .or_else(command::interruption);
    if let Some(signal) = interrupted {
        receipt["interrupted_by"] = command::signal_name(signal).into();
    }
    if let Some(error) = error.downcast_ref::<CommandFailure>() {
        for (key, bytes) in [
            ("command_stdout", &error.stdout),
            ("command_stderr", &error.stderr),
        ] {
            if let Some(bytes) = bytes {
                receipt[key] = String::from_utf8_lossy(bytes).into_owned().into();
            }
        }
        if !error.cleanup_errors.is_empty() {
            receipt["command_cleanup_errors"] = json!(error.cleanup_errors);
        }
    }
}

fn retain_storage_failure(receipt: &mut Value, error: &dyn std::error::Error) {
    if receipt.get("receipt_errors").is_none() {
        receipt["receipt_errors"] = json!([]);
    }
    receipt["receipt_errors"]
        .as_array_mut()
        .expect("receipt error array")
        .push(error.to_string().into());
    eprintln!("Receipt publication failed: {error}");
}

pub fn run_attempt(
    operations: &mut impl AttemptOperations,
    environment: &str,
    canister: &str,
    receipt: &mut Value,
) -> Result<()> {
    // No runtime is owned until startup is dispatched; an initial IO refusal ends here.
    attempt_checkpoint(operations, receipt, Some("starting"))?;
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
        receipt["network_cleanup"] = "pending".into();
        let checkpoint_error =
            attempt_checkpoint(operations, receipt, Some("stopping_network")).err();
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
        receipt["network_cleanup"] = if stop.is_ok() { "stopped" } else { "failed" }.into();
        if let Some(error) = stop.err().or(checkpoint_error) {
            receipt["status"] = "failed".into();
            if receipt.get("failed_phase").is_none() {
                receipt["failed_phase"] = receipt["phase"].clone();
            }
            receipt["cleanup_error"] = error.to_string().into();
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
    if verified && receipt["status"] == "running" {
        receipt["status"] = "passed".into();
    }
    receipt["finished_at"] = utc_now()?.into();
    if let Err(storage) = attempt_checkpoint(operations, receipt, Some("finished")) {
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
    receipt: &mut Value,
    startup_attempted: &mut bool,
) -> Result<()> {
    let canister = if environment == "local" {
        attempt_checkpoint(operations, receipt, Some("checking_network"))?;
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
        attempt_checkpoint(operations, receipt, Some("starting_network"))?;
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
        attempt_checkpoint(operations, receipt, Some("reading_network"))?;
        receipt["network"] = serde_json::from_str(&operations.icp(
            &["network", "status", "local", "--json"],
            &CommandOptions::default(),
        )?)?;
        receipt["api_endpoint"] = receipt["network"]["api_url"].clone();
        if receipt["api_endpoint"].as_str().is_none() {
            return Err(invalid("network discovery omitted api_url"));
        }
        attempt_checkpoint(operations, receipt, Some("deploying"))?;
        let deployment = serde_json::from_str(&operations.icp(
            &["deploy", "-e", "local", "--json"],
            &CommandOptions::default(),
        )?)?;
        deployed_canister(&deployment)?
    } else {
        receipt["api_endpoint"] = "https://icp-api.io/".into();
        attempt_checkpoint(operations, receipt, Some("building"))?;
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
