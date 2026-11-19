//! Migration-owned mainnet admission and bounded observation policy.

use crate::{Failure, FailureKind, Observation, Outcome};
use ic_platform::{
    certificate::{PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits},
    native::{ReadClient, ReadConfig},
};
use ic_platform_protocol::candid::Limits;
use ic_platform_types::Principal;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::Instant;

pub(crate) const ENDPOINT: &str = "https://ic0.app";
pub(crate) const ARGUMENT_BYTES: usize = 1024;
pub(crate) const OVERALL: Duration = Duration::from_secs(60);
pub(crate) const LIMITS: Limits = Limits {
    bytes: 1024 * 1024,
    header: 64 * 1024,
    types: 256,
    work: 4_000_000,
    allocation: 4 * 1024 * 1024,
    vector: 4096,
    skipping: 512 * 1024,
};
const LEAF: Duration = Duration::from_secs(20);

pub(crate) trait Source {
    async fn query(
        &self,
        target: Principal,
        method: &'static str,
        args: &[u8],
    ) -> Result<Vec<u8>, Failure>;
}

pub(crate) struct Mainnet(ReadClient);
impl Mainnet {
    pub(crate) fn new() -> Result<Self, Failure> {
        ReadClient::new(ReadConfig {
            endpoint: ENDPOINT.into(),
            root: PreparedRootKey::from_der(include_bytes!("root.der"))
                .expect("pinned mainnet root"),
            root_subnet: "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe"
                .parse()
                .expect("pinned mainnet subnet"),
            message_bytes: LIMITS.bytes,
            request_timeout: LEAF,
            verification: VerificationLimits {
                tree_nodes: 16384,
                tree_depth: 64,
                delegation_bytes: 131072,
                canister_ranges: 4096,
            },
        })
        .map(Self)
        .map_err(|e| Failure::new(FailureKind::Read, e.to_string()))
    }
}

impl Source for Mainnet {
    async fn query(
        &self,
        target: Principal,
        method: &'static str,
        args: &[u8],
    ) -> Result<Vec<u8>, Failure> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|n| u64::try_from(n.as_nanos()).ok())
            .ok_or_else(|| Failure::new(FailureKind::Clock, "Unix nanoseconds unavailable"))?;
        let expiry = now
            .checked_add(300_000_000_000)
            .ok_or_else(|| Failure::new(FailureKind::Clock, "ingress expiry overflow"))?;
        let window = TimeWindow {
            reference_ns: now,
            max_age_ns: 300_000_000_000,
            max_future_skew_ns: 60_000_000_000,
        };
        self.0
            .query(
                target,
                method,
                args,
                expiry,
                TimePolicy {
                    certificate: window,
                    delegation: window,
                },
            )
            .await
            .map_err(|e| Failure::new(FailureKind::Read, e.to_string()))
    }
}

fn now_ms() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|n| n.as_millis().try_into().ok())
}

pub(crate) async fn observe(
    source: &impl Source,
    target: Principal,
    method: &'static str,
    args: Result<Vec<u8>, Failure>,
    deadline: Instant,
) -> Observation<Vec<u8>> {
    let started = now_ms();
    let result = if started.is_none() {
        Err(Failure::new(
            FailureKind::Clock,
            "Unix observation time unavailable",
        ))
    } else if Instant::now() >= deadline {
        Err(Failure::new(
            FailureKind::Deadline,
            "collection deadline reached before query",
        ))
    } else {
        match args {
            Err(error) => Err(error),
            Ok(args) => tokio::time::timeout_at(
                deadline.min(Instant::now() + LEAF),
                source.query(target, method, &args),
            )
            .await
            .unwrap_or_else(|_| {
                Err(Failure::new(
                    FailureKind::Deadline,
                    "read deadline exceeded",
                ))
            }),
        }
    };
    let authenticated = result.is_ok();
    let finished = now_ms();
    let result = if finished.is_none() {
        Err(Failure::new(
            FailureKind::Clock,
            "Unix completion time unavailable",
        ))
    } else {
        result
    };
    Observation {
        method,
        started_at_unix_ms: started,
        finished_at_unix_ms: finished,
        query_signature_verified: authenticated,
        result_certified: false,
        outcome: match result {
            Ok(value) => Outcome::Read { value },
            Err(error) => Outcome::Failed { error },
        },
    }
}

pub(crate) fn encode(e: ic_platform_protocol::sns::EncodeError) -> Failure {
    Failure::new(FailureKind::Encode, e.to_string())
}

pub(crate) fn decode(e: ic_platform_protocol::candid::Error) -> Failure {
    Failure::new(FailureKind::Decode, e.to_string())
}

pub(crate) fn admission(message: &str) -> Failure {
    Failure::new(FailureKind::Admission, message)
}
