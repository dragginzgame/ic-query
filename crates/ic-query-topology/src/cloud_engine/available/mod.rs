//! Bounded public availability traversal with explicit per-page evidence.

mod source;
#[cfg(test)]
mod tests;
mod types;

use super::{CANISTER, Observation};
use crate::mainnet;
use ic_platform_protocol::{candid as decoder, cloud_engine as wire};
use ic_platform_types::Principal;
use std::{collections::BTreeSet, time::Duration};
use tokio::time::Instant;

use source::Source;
pub use types::*;

const STATUS_METHOD: &str = "registryCopyStatus";
const PAGE_METHOD: &str = "listAvailableNodesPage";
const PAGE_SIZE: usize = 100;

/// Default traversal budget: ten pages, each admitting at most 100 nodes.
pub const DEFAULT_PAGE_LIMIT: u16 = 10;

/// Largest caller-selected traversal budget; the 60-second deadline still applies.
pub const MAX_PAGE_LIMIT: u16 = 100;
const OVERALL: Duration = Duration::from_secs(60);
const LIMITS: decoder::Limits = decoder::Limits {
    bytes: 512 * 1024,
    header: 64 * 1024,
    types: 256,
    work: 2_000_000,
    allocation: 4 * 1024 * 1024,
    vector: 1024,
    skipping: 128 * 1024,
};

/// Read source-copy status and public new-engine eligible nodes, page by page.
///
/// Requests 100 nodes per page, at most ten pages/1,000 nodes and 60 seconds total,
/// with 20-second leaf reads and a 1 MiB HTTP cap. No owner-specific rejoining,
/// reservation, retry, history or background task is added. Requires Tokio IO/time.
/// Each page is read afresh: equal Registry versions and exhausted pagination do
/// not establish an atomic inventory, live capacity or verified TEE protection.
///
/// # Errors
/// A failed initial status observation returns a typed error without a report.
/// After status succeeds, every page failure or traversal limit returns a partial
/// report retaining status and previously admitted pages. Cancellation drops the
/// pending read and starts no later request.
pub async fn fetch() -> Result<AvailableNodesReport, AvailableError> {
    fetch_with_page_limit(DEFAULT_PAGE_LIMIT).await
}

/// Read public candidates with an explicit budget of 1–100 pages.
///
/// Each page admits at most 100 nodes, so the largest budget allows 10,000 nodes.
/// The 60-second overall deadline, source checks and partial-output rules of
/// [`fetch`] remain unchanged. This starts a fresh traversal, not a resumed snapshot.
///
/// # Errors
/// Refuses an invalid page budget before Platform construction or network IO.
/// Initial status failures are returned; subsequent page failures retain a report.
pub async fn fetch_with_page_limit(
    page_limit: u16,
) -> Result<AvailableNodesReport, AvailableError> {
    if !(1..=MAX_PAGE_LIMIT).contains(&page_limit) {
        return Err(AvailableError::InvalidPageLimit {
            requested: page_limit,
        });
    }

    collect(&source::MainnetSource(mainnet::client()?), page_limit).await
}

async fn collect(
    source: &impl Source,
    page_limit: u16,
) -> Result<AvailableNodesReport, AvailableError> {
    let node_limit = usize::from(page_limit) * PAGE_SIZE;
    let started = Instant::now();
    let deadline = started + OVERALL;
    let started_at_unix_seconds = mainnet::observed_at()?;
    let registry_copy = tokio::time::timeout_at(deadline, async {
        let observation = source.status().await?;
        let status = wire::decode_registry_copy_status_response(&observation.bytes, LIMITS)?;

        Ok::<_, AvailableError>(RegistryCopyObservation {
            source_method: STATUS_METHOD,
            observed_at_unix_seconds: observation.observed_at,
            version: status.version,
            latest: status.latest,
            caught_up: status.caught_up,
        })
    })
    .await
    .map_err(|_| AvailableError::OverallDeadline)??;
    let mut pages = Vec::new();
    let mut nodes = BTreeSet::new();
    let mut cursors = BTreeSet::new();
    let mut start = None;
    let mut version = None;
    let stop = loop {
        if Instant::now() >= deadline {
            break TraversalStop::OverallDeadline { start };
        }
        let observation = match tokio::time::timeout_at(deadline, source.page(start)).await {
            Err(_) => break TraversalStop::OverallDeadline { start },
            Ok(Err(error)) => break failed(start, None, None, error),
            Ok(Ok(observation)) => observation,
        };
        let observed_at = Some(observation.observed_at);
        let page = match decode_page(&observation) {
            Ok(page) => page,
            Err(error) => break failed(start, observed_at, None, error),
        };
        if let Err(error) = admit(&page, version, &nodes, &cursors, node_limit) {
            break failed(
                start,
                observed_at,
                Some(page.registry_version),
                error.into(),
            );
        }
        version = Some(page.registry_version);
        let next = page.next;
        nodes.extend(page.nodes.iter().map(|node| node.node_id));
        if let Some(cursor) = next {
            cursors.insert(cursor);
        }
        pages.push(AvailablePage {
            start,
            next,
            registry_version: page.registry_version,
            observed_at_unix_seconds: observation.observed_at,
            nodes: page
                .nodes
                .into_iter()
                .map(|node| AvailableNode {
                    node_id: node.node_id,
                    node_type: node.node_type,
                    operator_id: node.operator,
                    provider_id: node.provider,
                    data_center: node.dc,
                    sev_enabled: node.sev_enabled,
                })
                .collect(),
        });
        if next.is_none() {
            break TraversalStop::Exhausted;
        }
        if nodes.len() == node_limit {
            break TraversalStop::NodeLimit;
        }
        if pages.len() == usize::from(page_limit) {
            break TraversalStop::PageLimit;
        }
        start = next;
    };

    Ok(AvailableNodesReport {
        schema: 1,
        page_limit,
        node_limit,
        started_at_unix_seconds,
        elapsed: started.elapsed(),
        endpoint: mainnet::ENDPOINT,
        source_canister: CANISTER,
        source_method: PAGE_METHOD,
        rejoining: None,
        query_signature_verified: true,
        result_certified: false,
        atomic_snapshot: false,
        point_in_time_inventory_complete: false,
        pagination_exhausted: matches!(stop, TraversalStop::Exhausted),
        registry_copy,
        pages,
        stop,
    })
}

fn failed(
    start: Option<Principal>,
    observed_at_unix_seconds: Option<u64>,
    observed_registry_version: Option<u64>,
    error: AvailableError,
) -> TraversalStop {
    TraversalStop::Failed {
        start,
        observed_at_unix_seconds,
        observed_registry_version,
        error,
    }
}

fn decode_page(observation: &Observation) -> Result<wire::AvailableNodesPage, AvailableError> {
    match wire::decode_list_available_nodes_page_response(&observation.bytes, LIMITS)? {
        wire::AvailableNodesPageResult::Ok(Some(page)) => Ok(page),
        wire::AvailableNodesPageResult::Ok(None) => Err(AvailableError::CopyNotReady),
        wire::AvailableNodesPageResult::Err(error) => Err(AvailableError::Application(error)),
    }
}

fn admit(
    page: &wire::AvailableNodesPage,
    version: Option<u64>,
    seen: &BTreeSet<Principal>,
    cursors: &BTreeSet<Principal>,
    node_limit: usize,
) -> Result<(), PageRefusal> {
    if let Some(expected) = version
        && page.registry_version != expected
    {
        return Err(PageRefusal::VersionChanged {
            expected,
            observed: page.registry_version,
        });
    }
    for (field, actual, maximum) in [
        ("page_nodes", page.nodes.len(), PAGE_SIZE),
        ("total_nodes", seen.len() + page.nodes.len(), node_limit),
    ] {
        if actual > maximum {
            return Err(PageRefusal::Limit {
                field,
                actual,
                maximum,
            });
        }
    }
    if let Some(cursor) = page.next
        && cursors.contains(&cursor)
    {
        return Err(PageRefusal::RepeatedCursor { cursor });
    }
    // Validate the entire page before committing any identity to traversal state.
    let mut page_nodes = BTreeSet::new();
    for node in &page.nodes {
        for (field, id) in [
            ("node", node.node_id),
            ("operator", node.operator),
            ("provider", node.provider),
        ] {
            if id.as_slice().is_empty() {
                return Err(PageRefusal::EmptyIdentity { field });
            }
        }
        if seen.contains(&node.node_id) || !page_nodes.insert(node.node_id) {
            return Err(PageRefusal::DuplicateNode {
                node_id: node.node_id,
            });
        }
        if let Some(dc) = &node.dc
            && dc.len() > 256
        {
            return Err(PageRefusal::DataCenter {
                node_id: node.node_id,
                bytes: dc.len(),
            });
        }
    }

    Ok(())
}
