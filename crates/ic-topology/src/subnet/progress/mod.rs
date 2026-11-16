//! Two authenticated subnet state times, without an uptime or completion verdict.
use crate::{ReadError, mainnet};
use ic_platform_next::certificate::{
    PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits, VerifyError,
    authenticate_subnet_time_from_certificate,
};
use ic_platform_types::Principal;
use serde::{Serialize, Serializer};
use std::{
    future::Future,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tokio::time::Instant;

// Fixed mainnet root/subnet binding, never derived from the requested ID or reply.
const ROOT_SUBNET: &str = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";
const MAX_AGE_NS: u64 = 300_000_000_000;
const FUTURE_SKEW_NS: u64 = 60_000_000_000;
const LIMITS: VerificationLimits = VerificationLimits {
    tree_nodes: 4096,
    tree_depth: 64,
    delegation_bytes: 128 * 1024,
    canister_ranges: 0,
};

/// One admitted signing time with explicit source and freshness policy.
#[derive(Debug, Serialize)]
pub struct StateTimeObservation {
    /// Selected subnet authenticated by Platform.
    pub subnet_id: Principal,
    /// Raw signing-certificate Unix nanoseconds.
    pub certified_time_nanoseconds: u64,
    /// Separate delegation time; absent for the trusted root subnet.
    pub delegation_time_nanoseconds: Option<u64>,
    /// Local completion/verification reference in Unix nanoseconds.
    pub observed_at_unix_nanoseconds: u64,
    /// Fixed admitted endpoint.
    pub endpoint: &'static str,
    /// Subnet-scoped read-state path.
    pub source_path: String,
    /// Checked leaf path, shared by all read-state certificates.
    pub state_tree_path: &'static str,
    /// Inclusive maximum age, applied to signing and delegation timestamps.
    pub maximum_age_nanoseconds: u64,
    /// Inclusive future skew, applied to signing and delegation timestamps.
    pub maximum_future_skew_nanoseconds: u64,
    /// Platform authenticated time, signatures and exact subnet authority.
    pub state_certificate_verified: bool,
    /// False: Agent returns a parsed certificate, not original encoding bytes.
    pub original_certificate_bytes_retained: bool,
}

/// Ordering of the two observed state times; no operational diagnosis.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StateTimeChange {
    /// Second time is larger.
    Advanced {
        /// Exact timestamp difference, not a throughput measurement.
        by_nanoseconds: u64,
    },
    /// Same time may represent repeated state.
    Unchanged,
    /// Earlier state was served second; no subtraction or reset inference.
    Regressed,
}

/// Retained second-read result; failure is unavailable evidence, not downtime.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LaterObservation {
    /// Successfully authenticated later read.
    Read {
        /// Independent original observation.
        observation: StateTimeObservation,
        /// Ordering relative to the retained first observation.
        change: StateTimeChange,
    },
    /// First observation remains available when this read fails.
    Failed {
        /// Typed cause; JSON retains a category, not arbitrary diagnostics.
        #[serde(serialize_with = "read_error")]
        error: StateTimeError,
    },
}
/// One bounded pair, preserving partial evidence.
#[derive(Debug, Serialize)]
pub struct SubnetProgressReport {
    /// Query-owned schema.
    pub schema: u8,
    /// Selected identity common to admitted observations.
    pub subnet_id: Principal,
    /// Local wait after the first read, not a certified interval.
    pub interval_seconds: u64,
    /// First authenticated observation.
    pub before: StateTimeObservation,
    /// Second observation or failure.
    pub after: LaterObservation,
    /// Total monotonic acquisition time.
    pub elapsed: Duration,
    /// Both reads succeeded, regardless of their ordering.
    pub collection_complete: bool,
}
/// Failed acquisition; never a subnet health classification.
#[derive(Debug, Error)]
pub enum StateTimeError {
    /// Shared transport/clock/deadline error.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// Supplier refused root, authority, signature, bounds or freshness.
    #[error("subnet time authentication failed: {0}")]
    Authentication(#[from] VerifyError),
    /// Principal could not cross an external identity boundary.
    #[error("invalid subnet identity: {0}")]
    Identity(String),
    /// Local Unix nanoseconds do not fit the source's u64 time contract.
    #[error("local Unix nanoseconds exceed u64")]
    ClockRange,
    /// Reader returned a different identity or unverified source.
    #[error("subnet time observation does not match the admitted source")]
    SourceMismatch,
}
/// Failures before a progress report can be anchored.
#[derive(Debug, Error)]
pub enum ProgressError {
    /// Invalid interval rejected before IO.
    #[error("progress interval must be between 1 and 60 seconds")]
    InvalidInterval,
    /// No first authenticated observation was obtained.
    #[error("first state-time observation failed: {0}")]
    First(#[source] StateTimeError),
}

/// Read signing time twice, with one wait of 1–60 seconds.
///
/// Uses mainnet admission, 20 seconds per read and a 1 MiB HTTP ceiling. Both
/// certificate times must be within 300 seconds past/60 seconds future of local
/// verification time. Agent outer decoding precedes Platform's bounded traversal;
/// original encoding bytes are unavailable. No retries, tasks or persistence.
/// At most 100 asynchronous seconds plus bounded verification and scheduling.
/// Requires Tokio IO/time. Cancellation starts no subsequent acquisition.
///
/// # Errors
/// Rejects invalid interval or failed first observation. Later failures are retained.
pub async fn fetch(
    subnet_id: Principal,
    interval_seconds: u64,
) -> Result<SubnetProgressReport, ProgressError> {
    sample(subnet_id, interval_seconds, || observe(subnet_id)).await
}

async fn observe(subnet_id: Principal) -> Result<StateTimeObservation, StateTimeError> {
    let agent = mainnet::agent()?;
    let external = subnet_id
        .as_slice()
        .try_into()
        .map_err(|e| StateTimeError::Identity(format!("{e}")))?;
    let certificate =
        mainnet::read(agent.read_subnet_state_raw(vec![vec!["time".into()]], external)).await?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(ReadError::Clock)?;
    let reference_ns = u64::try_from(now.as_nanos()).map_err(|_| StateTimeError::ClockRange)?;
    let root = PreparedRootKey::from_der(&agent.read_root_key())?;
    let requested = subnet_id
        .to_text()
        .parse()
        .map_err(|e| StateTimeError::Identity(format!("{e}")))?;
    let root_subnet = ROOT_SUBNET
        .parse()
        .map_err(|e| StateTimeError::Identity(format!("{e}")))?;
    let window = TimeWindow {
        reference_ns,
        max_age_ns: MAX_AGE_NS,
        max_future_skew_ns: FUTURE_SKEW_NS,
    };
    let evidence = authenticate_subnet_time_from_certificate(
        &certificate,
        &root,
        requested,
        Some(root_subnet),
        TimePolicy {
            certificate: window,
            delegation: window,
        },
        LIMITS,
    )?;
    Ok(StateTimeObservation {
        subnet_id,
        certified_time_nanoseconds: evidence.time_ns(),
        delegation_time_nanoseconds: evidence.delegation().map(|(_, time)| time),
        observed_at_unix_nanoseconds: reference_ns,
        endpoint: mainnet::ENDPOINT,
        source_path: format!("/api/v3/subnet/{subnet_id}/read_state"),
        state_tree_path: "/time",
        maximum_age_nanoseconds: MAX_AGE_NS,
        maximum_future_skew_nanoseconds: FUTURE_SKEW_NS,
        state_certificate_verified: true,
        original_certificate_bytes_retained: false,
    })
}
fn admit(
    value: StateTimeObservation,
    subnet: Principal,
) -> Result<StateTimeObservation, StateTimeError> {
    if value.subnet_id != subnet || !value.state_certificate_verified {
        return Err(StateTimeError::SourceMismatch);
    }
    Ok(value)
}
async fn sample<F, Fut>(
    subnet_id: Principal,
    interval_seconds: u64,
    mut read: F,
) -> Result<SubnetProgressReport, ProgressError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<StateTimeObservation, StateTimeError>>,
{
    if !(1..=60).contains(&interval_seconds) {
        return Err(ProgressError::InvalidInterval);
    }
    let started = Instant::now();
    let before = read()
        .await
        .and_then(|v| admit(v, subnet_id))
        .map_err(ProgressError::First)?;
    tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
    let after = match read().await.and_then(|v| admit(v, subnet_id)) {
        Ok(observation) => {
            let change = match observation
                .certified_time_nanoseconds
                .cmp(&before.certified_time_nanoseconds)
            {
                std::cmp::Ordering::Greater => StateTimeChange::Advanced {
                    by_nanoseconds: observation.certified_time_nanoseconds
                        - before.certified_time_nanoseconds,
                },
                std::cmp::Ordering::Equal => StateTimeChange::Unchanged,
                std::cmp::Ordering::Less => StateTimeChange::Regressed,
            };
            LaterObservation::Read {
                observation,
                change,
            }
        }
        Err(error) => LaterObservation::Failed { error },
    };
    Ok(SubnetProgressReport {
        schema: 1,
        subnet_id,
        interval_seconds,
        collection_complete: matches!(&after, LaterObservation::Read { .. }),
        before,
        after,
        elapsed: started.elapsed(),
    })
}
fn read_error<S: Serializer>(error: &StateTimeError, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(match error {
        StateTimeError::Read(ReadError::Deadline) => "read_deadline",
        StateTimeError::Read(ReadError::Transport(_)) => "transport",
        StateTimeError::Read(ReadError::Clock(_)) | StateTimeError::ClockRange => "clock",
        StateTimeError::Authentication(_) => "authentication",
        StateTimeError::Identity(_) | StateTimeError::SourceMismatch => "source_identity",
    })
}
#[cfg(test)]
mod tests;
