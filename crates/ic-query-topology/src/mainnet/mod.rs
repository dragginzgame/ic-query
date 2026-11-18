//! Shared admission and limits for the implemented mainnet reports.

#[cfg(test)]
mod tests;

use std::{
    future::Future,
    time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH},
};

use ic_platform::{
    certificate::{PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits},
    native::{ReadClient, ReadConfig, ReadError as NativeError},
};
use ic_platform_protocol::registry as protobuf;
use ic_platform_types::Principal;
use thiserror::Error;

pub(crate) const ENDPOINT: &str = "https://ic0.app";
pub(crate) const ROOT_SUBNET: &str =
    "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";
pub(crate) const REGISTRY: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
pub(crate) const REGISTRY_METHOD: &str = "get_value";
pub(crate) const REGISTRY_LIMITS: protobuf::Limits = protobuf::Limits {
    bytes: 64 * 1024,
    fields: 4096,
    allocation: 128 * 1024,
    items: 1024,
};
const DEADLINE: Duration = Duration::from_secs(20);
const HTTP_BODY_BYTES: usize = 1024 * 1024;
const INGRESS_EXPIRY: Duration = Duration::from_secs(300);

/// Shared failures from bounded mainnet reads and observation timing.
#[derive(Debug, Error)]
pub enum ReadError {
    /// Platform construction, transport, certificate or query verification failed.
    #[error("mainnet transport or verification failed: {0}")]
    Transport(#[source] Box<NativeError>),
    /// The asynchronous operation exceeded its overall deadline.
    #[error("mainnet read exceeded its 20-second deadline")]
    Deadline,
    /// The local clock could not supply a Unix observation time.
    #[error("unable to record observation time: {0}")]
    Clock(#[source] SystemTimeError),
    /// The local clock or ingress expiry does not fit protocol nanoseconds.
    #[error("local clock exceeds protocol nanosecond range")]
    ClockRange,
}

pub(crate) fn client() -> Result<ReadClient, ReadError> {
    ReadClient::new(ReadConfig {
        endpoint: ENDPOINT.into(),
        root: PreparedRootKey::from_der(include_bytes!("root.der")).expect("pinned root"),
        root_subnet: ROOT_SUBNET.parse().expect("pinned root subnet"),
        message_bytes: HTTP_BODY_BYTES,
        request_timeout: DEADLINE,
        verification: VerificationLimits {
            tree_nodes: 16384,
            tree_depth: 64,
            delegation_bytes: 131072,
            canister_ranges: 4096,
        },
    })
    .map_err(|e| ReadError::Transport(Box::new(e)))
}

fn context() -> Result<(u64, TimePolicy), ReadError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(ReadError::Clock)?;
    context_at(now)
}

fn context_at(now: Duration) -> Result<(u64, TimePolicy), ReadError> {
    let now = u64::try_from(now.as_nanos()).map_err(|_| ReadError::ClockRange)?;
    let expiry = now
        .checked_add(INGRESS_EXPIRY.as_nanos() as u64)
        .ok_or(ReadError::ClockRange)?;
    let window = TimeWindow {
        reference_ns: now,
        max_age_ns: 300_000_000_000,
        max_future_skew_ns: 60_000_000_000,
    };
    Ok((
        expiry,
        TimePolicy {
            certificate: window,
            delegation: window,
        },
    ))
}

pub(crate) async fn query(
    client: &ReadClient,
    target: Principal,
    method: &str,
    args: &[u8],
) -> Result<Vec<u8>, ReadError> {
    let (expiry, policy) = context()?;
    read(client.query(target, method, args, expiry, policy)).await
}

pub(crate) async fn subnet(
    client: &ReadClient,
    target: Principal,
) -> Result<ic_platform::native::SubnetObservation, ReadError> {
    let (expiry, policy) = context()?;
    read(client.read_subnet(target, expiry, policy)).await
}

pub(crate) async fn read<T>(
    operation: impl Future<Output = Result<T, NativeError>>,
) -> Result<T, ReadError> {
    tokio::time::timeout(DEADLINE, operation)
        .await
        .map_err(|_| ReadError::Deadline)?
        .map_err(|error| ReadError::Transport(Box::new(error)))
}

pub(crate) fn observed_at() -> Result<u64, ReadError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(ReadError::Clock)
}
