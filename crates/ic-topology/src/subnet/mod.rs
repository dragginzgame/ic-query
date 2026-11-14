//! Subnet inventory, observed node identities and certified resource values.
//!
//! These observations do not establish complete Registry membership, node
//! liveness, allocatable capacity, hardware capabilities or authoritative routing.

use ic_platform::agent::SubnetType;
use ic_platform_types::Principal;
use serde::Serialize;
use thiserror::Error;

use crate::{ReadError, mainnet};

pub mod activity;
pub mod configuration;
pub mod list;
pub mod membership;
pub mod metrics;

const MAX_NODES: usize = 1024;
const MAX_SUBNET_TYPE_BYTES: usize = 128;

/// One selected subnet's observed type and node-key identities.
#[derive(Debug, Serialize)]
pub struct SubnetReport {
    /// Query-owned report schema; remains 1 before 1.0.
    pub schema: u8,
    /// Explicitly selected subnet identity, checked against the source result.
    pub subnet_id: Principal,
    /// Sorted identities with node-key leaves returned by the authenticated source.
    pub observed_node_ids: Vec<Principal>,
    /// Subnet-certified type, preserving unknown strings; None means absent.
    /// Does not establish independent Registry agreement or deployment permission.
    pub observed_subnet_type: Option<String>,
    /// Source, verification and coverage claims.
    pub evidence: SubnetEvidence,
}

/// Evidence for the type and node-key identities returned by the selected subnet.
#[derive(Debug, Serialize)]
pub struct SubnetEvidence {
    /// Admitted mainnet HTTPS endpoint.
    pub endpoint: &'static str,
    /// Subnet-scoped read-state HTTP path.
    pub source_path: String,
    /// Local completion time, in raw Unix seconds; not the certificate timestamp.
    pub observed_at_unix_seconds: u64,
    /// Agent verified the state certificate, subnet delegation and timestamp policy.
    pub state_certificate_verified: bool,
    /// False: Agent's projected node-key list does not establish complete membership.
    pub membership_complete: bool,
    /// None: this source supplies no Registry version.
    pub registry_version: Option<u64>,
}

/// Refusals from a subnet observation; no partial result is returned.
#[derive(Debug, Error)]
pub enum SubnetError {
    /// Mainnet transport, verification, deadline or observation-clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The selected principal could not cross the Agent identity boundary.
    #[error("invalid subnet identity: {0}")]
    SourceIdentity(String),
    /// The source returned a different subnet than requested.
    #[error("source subnet {returned} differs from requested subnet {requested}")]
    IdentityMismatch {
        /// Requested subnet.
        requested: Principal,
        /// Unexpected source identity.
        returned: Principal,
    },
    /// The source projection exceeded the report's bounded inventory scope.
    #[error("subnet observation exceeds the 1024-node limit")]
    NodeLimit,
    /// The source supplied no node-key evidence; do not report an empty subnet.
    #[error("subnet source returned no node-key identities")]
    MissingNodes,
    /// Duplicate identities make the projection ambiguous.
    #[error("subnet source repeated a node identity")]
    DuplicateNode,
    /// The observed type exceeded the retained UTF-8 byte budget.
    #[error("subnet type exceeds the 128-byte limit")]
    SubnetTypeLimit,
}

/// Observe type and node-key identities in one subnet's mainnet certified state.
///
/// Requires a Tokio runtime with IO and time enabled. Uses a fresh Agent fetch,
/// the shared 20-second deadline and 1 MiB HTTP ceiling. Retains at most 1024
/// node identities and 128 UTF-8 bytes of subnet type. An absent type remains None;
/// unknown values are preserved. Returned canister ranges and subnet keys are
/// deliberately excluded: they do not establish NNS-authorized routing or trust.
///
/// # Errors
/// Returns typed read failures, identity mismatch, missing/duplicate node evidence
/// or a node-count/type-byte limit refusal. A failure does not prove subnet downtime.
pub async fn fetch(subnet_id: Principal) -> Result<SubnetReport, SubnetError> {
    let agent = mainnet::agent()?;
    let external_id = subnet_id
        .as_slice()
        .try_into()
        .map_err(|error| SubnetError::SourceIdentity(format!("{error}")))?;
    let observation = mainnet::read(agent.fetch_subnet_by_id(&external_id)).await?;
    project(
        subnet_id,
        Principal::from_slice(observation.id().as_slice()),
        observation
            .iter_nodes()
            .map(|node| Principal::from_slice(node.as_slice())),
        mainnet::observed_at()?,
        observation.subnet_type(),
    )
}

fn project(
    requested: Principal,
    returned: Principal,
    nodes: impl IntoIterator<Item = Principal>,
    observed_at: u64,
    subnet_type: Option<&SubnetType>,
) -> Result<SubnetReport, SubnetError> {
    if returned != requested {
        return Err(SubnetError::IdentityMismatch {
            requested,
            returned,
        });
    }
    let mut nodes: Vec<_> = nodes.into_iter().take(MAX_NODES + 1).collect();
    if nodes.len() > MAX_NODES {
        return Err(SubnetError::NodeLimit);
    }
    if nodes.is_empty() {
        return Err(SubnetError::MissingNodes);
    }
    nodes.sort_unstable();
    if nodes.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(SubnetError::DuplicateNode);
    }
    let subnet_type = subnet_type.map(|value| match value {
        SubnetType::System => "system",
        SubnetType::Application => "application",
        SubnetType::VerifiedApplication => "verified_application",
        SubnetType::CloudEngine => "cloud_engine",
        SubnetType::Unknown(raw) => raw.as_str(),
    });
    if subnet_type.is_some_and(|value| value.len() > MAX_SUBNET_TYPE_BYTES) {
        return Err(SubnetError::SubnetTypeLimit);
    }
    Ok(SubnetReport {
        schema: 1,
        subnet_id: requested,
        observed_node_ids: nodes,
        observed_subnet_type: subnet_type.map(str::to_owned),
        evidence: SubnetEvidence {
            endpoint: mainnet::ENDPOINT,
            source_path: format!("/api/v3/subnet/{requested}/read_state"),
            observed_at_unix_seconds: observed_at,
            state_certificate_verified: true,
            membership_complete: false,
            registry_version: None,
        },
    })
}

#[cfg(test)]
mod tests;
