//! Selected public engine-to-subnet/operator/node evidence for Canic planning.

use super::{CANISTER, Observation};
use crate::mainnet;
use ic_platform_protocol::{candid as decoder, cloud_engine as wire};
use ic_platform_types::Principal;
use std::{collections::BTreeSet, future::Future};
use tokio::time::Instant;

mod types;
pub use types::*;

const METHOD: &str = "getEngineViews";
const MAX_NODES: usize = 128;
const LIMITS: decoder::Limits = decoder::Limits {
    bytes: 512 * 1024,
    header: 64 * 1024,
    types: 256,
    work: 2_000_000,
    allocation: 4 * 1024 * 1024,
    vector: 1024,
    skipping: 128 * 1024,
};

/// Read one selected engine's public control-plane view.
///
/// Parse an [`EngineId`] before calling. Uses one anonymous verified mainnet query,
/// a 20-second read deadline and a 1 MiB HTTP ceiling. No retry, scan or operator
/// detail query is added. Cancellation drops the pending read. Requires Tokio IO/time.
/// Unknown/hidden IDs yield a successful `NotReturned` report. Registry-copy absence
/// remains distinct from empty membership; no copy version or freshness is inferred.
///
/// # Errors
/// Returns original transport/codec errors or typed record-admission refusals.
/// A read failure never becomes a successful empty result.
pub async fn fetch(engine_id: EngineId) -> Result<EngineReport, EngineError> {
    let agent = mainnet::agent()?;
    collect(engine_id, |argument| async move {
        super::query(&agent, METHOD, argument)
            .await
            .map_err(EngineError::Read)
    })
    .await
}

async fn collect<F, Fut>(engine_id: EngineId, query: F) -> Result<EngineReport, EngineError>
where
    F: FnOnce(Vec<u8>) -> Fut,
    Fut: Future<Output = Result<Observation, EngineError>>,
{
    let start = Instant::now();
    let started_at_unix_seconds = mainnet::observed_at()?;
    let argument = wire::encode_get_engine_views_args(&[engine_id.as_str().to_owned()], 512)
        .map_err(EngineError::Encode)?;
    let observation = query(argument).await?;
    let views = wire::decode_get_engine_views_response(&observation.bytes, LIMITS)
        .map_err(EngineError::Decode)?;
    let outcome = project(&engine_id, views)?;
    Ok(EngineReport {
        schema: 1,
        requested_engine_id: engine_id,
        started_at_unix_seconds,
        elapsed: start.elapsed(),
        atomic_snapshot: false,
        outcome,
        evidence: EngineEvidence {
            endpoint: mainnet::ENDPOINT,
            source_canister: CANISTER,
            source_method: METHOD,
            observed_at_unix_seconds: observation.observed_at,
            query_signature_verified: true,
            result_certified: false,
        },
    })
}

fn project(
    request: &EngineId,
    mut views: Vec<wire::EngineView>,
) -> Result<EngineOutcome, EngineRefusal> {
    bound("views", views.len(), 1)?;
    let Some(view) = views.pop() else {
        return Ok(EngineOutcome::NotReturned);
    };
    bound("id_bytes", view.id.len(), 256)?;
    if view.id != request.as_str() {
        return Err(EngineRefusal::WrongEngine);
    }
    let subnet_id = view.subnet_id;
    let operator_id = view.engine_operator_id;
    let node_ids = view.node_ids;
    for (field, value) in [
        ("subnet_id", subnet_id),
        ("operator_canister_id", operator_id),
    ] {
        if let Some(id) = value {
            identity(field, id)?;
        }
    }
    let record_set = nodes("record_node_ids", &node_ids)?;
    let mut membership = MembershipComparison::Unavailable;
    let registry_copy_subnet = if let Some(subnet) = view.subnet {
        if view.subnet_id.is_none() {
            return Err(EngineRefusal::MissingSubnetId);
        }
        bound("replica_version_bytes", subnet.replica_version.len(), 256)?;
        let copy_nodes = subnet.node_ids;
        let copy_set = nodes("registry_copy_node_ids", &copy_nodes)?;
        membership = MembershipComparison::Compared {
            record_only: record_set.difference(&copy_set).copied().collect(),
            registry_copy_only: copy_set.difference(&record_set).copied().collect(),
        };
        Some(RegistryCopySubnet {
            node_ids: copy_nodes,
            configured_replica_version: subnet.replica_version,
            halted: subnet.halted,
            maximum_state_size_bytes: subnet.maximum_state_size,
        })
    } else {
        None
    };
    Ok(EngineOutcome::Read {
        record: Box::new(EngineRecord {
            subnet_id,
            operator_canister_id: operator_id,
            record_node_ids: node_ids,
            deleted_at_unix_nanoseconds: view.deleted_at,
            registry_copy_subnet,
            membership,
        }),
    })
}

fn bound(field: &'static str, actual: usize, maximum: usize) -> Result<(), EngineRefusal> {
    if actual > maximum {
        return Err(EngineRefusal::Limit {
            field,
            actual,
            maximum,
        });
    }
    Ok(())
}

fn identity(field: &'static str, id: Principal) -> Result<(), EngineRefusal> {
    if id.as_slice().is_empty() {
        return Err(EngineRefusal::EmptyIdentity { field });
    }
    Ok(())
}

fn nodes(field: &'static str, nodes: &[Principal]) -> Result<BTreeSet<Principal>, EngineRefusal> {
    bound(field, nodes.len(), MAX_NODES)?;
    let mut set = BTreeSet::new();
    for &node_id in nodes {
        identity(field, node_id)?;
        if !set.insert(node_id) {
            return Err(EngineRefusal::DuplicateNode { field, node_id });
        }
    }
    Ok(set)
}

#[cfg(test)]
mod tests;
