//! Module: ic::source::replica_version
//!
//! Responsibility: Dashboard replica-version source contract, validation, and projection.
//! Does not own: HTTP transport, shared provenance, CLI paging, or text rendering.
//! Boundary: validates one bounded page or one exact release record without runtime claims.

use super::invalid_request;
use super::invalid_source;
use super::report_provenance;
use super::validate_canonical_principal;
use super::validate_offset_page;
use super::validate_provenance;
use crate::hex::is_lowercase_hex;
use crate::ic::IcHostError;
use crate::ic::IcReplicaVersionInfoReport;
use crate::ic::IcReplicaVersionInfoSourceData;
use crate::ic::IcReplicaVersionListQuery;
use crate::ic::IcReplicaVersionListReport;
use crate::ic::IcReplicaVersionListRow;
use crate::ic::IcReplicaVersionListSourceData;
use crate::ic::IcReplicaVersionStatus;
use crate::ic::IcReplicaVersionSubnetRollout;
use crate::ic::IcSourceRequest;
use crate::ic::MAX_IC_REPLICA_VERSION_PAGE_LIMIT;
use std::collections::HashSet;

///
/// IcReplicaVersionSource
///
/// Source contract for bounded official Dashboard replica-version reports.
///

pub trait IcReplicaVersionSource: Sync {
    /// Fetch one explicitly bounded release page without automatically paginating.
    fn fetch_replica_version_list<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcReplicaVersionListQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcReplicaVersionListSourceData, IcHostError>>;

    /// Fetch one exact release record by replica-version identifier.
    fn fetch_replica_version_info<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        replica_version_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<IcReplicaVersionInfoSourceData, IcHostError>>;
}

pub(in crate::ic) fn validate_replica_version_list_query(
    query: &IcReplicaVersionListQuery,
) -> Result<(), IcHostError> {
    if (1..=MAX_IC_REPLICA_VERSION_PAGE_LIMIT).contains(&query.limit) {
        return Ok(());
    }
    invalid_request(
        "query.limit",
        format!("must be between 1 and {MAX_IC_REPLICA_VERSION_PAGE_LIMIT}"),
    )
}

pub(in crate::ic) fn validate_replica_version_id(
    replica_version_id: &str,
) -> Result<(), IcHostError> {
    if replica_version_id.len() == 40 && is_lowercase_hex(replica_version_id) {
        return Ok(());
    }
    invalid_request(
        "replica_version_id",
        "must be exactly 40 lowercase hexadecimal characters",
    )
}

pub(in crate::ic) fn replica_version_list_report_from_source(
    request: &IcSourceRequest,
    query: &IcReplicaVersionListQuery,
    mut source: IcReplicaVersionListSourceData,
) -> Result<IcReplicaVersionListReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    super::validate_requested_query("replica-version", query, &source.query)?;
    super::validate_indexed_page_bounds(
        source.query.max_proposal_index,
        source.resolved_max_proposal_index,
        "resolved_max_proposal_index",
        source.rows.len(),
        source.query.limit,
        "replica-version",
    )?;
    let next_offset = validate_offset_page(
        source.query.offset,
        source.total_proposals,
        source.rows.len(),
        "total_proposals",
        "replica-version page offset overflows u64",
    )?;
    validate_list_rows(&mut source.rows)?;

    Ok(IcReplicaVersionListReport {
        provenance: report_provenance(source.source),
        query: source.query,
        resolved_max_proposal_index: source.resolved_max_proposal_index,
        total_proposals: source.total_proposals,
        next_offset,
        rows: source.rows,
    })
}

pub(in crate::ic) fn replica_version_info_report_from_source(
    request: &IcSourceRequest,
    requested_replica_version_id: &str,
    mut source: IcReplicaVersionInfoSourceData,
) -> Result<IcReplicaVersionInfoReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_release_identity(
        "replica_version_id",
        &source.release.replica_version_id,
        source.release.proposal_id,
    )?;
    if source.release.replica_version_id != requested_replica_version_id {
        return invalid_source(format!(
            "replica_version_id is {:?}, expected requested value {requested_replica_version_id:?}",
            source.release.replica_version_id
        ));
    }
    validate_subnet_rollouts(&mut source.release.subnets)?;

    Ok(IcReplicaVersionInfoReport {
        release: source.release,
        provenance: report_provenance(source.source),
    })
}

fn validate_list_rows(rows: &mut [IcReplicaVersionListRow]) -> Result<(), IcHostError> {
    let mut seen_proposals = HashSet::with_capacity(rows.len());
    let mut previous_timestamp = None;
    for row in rows.iter_mut() {
        validate_release_identity(
            "row.replica_version_id",
            &row.replica_version_id,
            row.proposal_id,
        )?;
        if !seen_proposals.insert(row.proposal_id) {
            return invalid_source(format!(
                "duplicate replica-version proposal id {}",
                row.proposal_id
            ));
        }
        if row.status == IcReplicaVersionStatus::Executed && row.executed_timestamp_seconds == 0 {
            return invalid_source(format!(
                "executed replica-version {} has zero execution timestamp",
                row.replica_version_id
            ));
        }
        if previous_timestamp.is_some_and(|previous| row.executed_timestamp_seconds > previous) {
            return invalid_source(
                "replica-version rows are not in descending execution-time order",
            );
        }
        previous_timestamp = Some(row.executed_timestamp_seconds);
        validate_subnet_rollouts(&mut row.subnets)?;
    }
    Ok(())
}

fn validate_release_identity(
    field: &'static str,
    replica_version_id: &str,
    proposal_id: u64,
) -> Result<(), IcHostError> {
    if replica_version_id.len() != 40 || !is_lowercase_hex(replica_version_id) {
        return invalid_source(format!(
            "{field} must be exactly 40 lowercase hexadecimal characters"
        ));
    }
    if proposal_id == 0 {
        return invalid_source("replica-version proposal_id must be positive");
    }
    Ok(())
}

fn validate_subnet_rollouts(
    subnets: &mut [IcReplicaVersionSubnetRollout],
) -> Result<(), IcHostError> {
    let mut seen_proposals = HashSet::with_capacity(subnets.len());
    for rollout in subnets.iter() {
        validate_canonical_principal("subnet.subnet_id", &rollout.subnet_id)?;
        if rollout.proposal_id == 0 {
            return invalid_source("Subnet rollout proposal_id must be positive");
        }
        if !seen_proposals.insert(rollout.proposal_id) {
            return invalid_source(format!(
                "duplicate Subnet rollout proposal id {}",
                rollout.proposal_id
            ));
        }
    }
    subnets.sort_unstable_by(|left, right| {
        left.executed_timestamp_seconds
            .cmp(&right.executed_timestamp_seconds)
            .then_with(|| left.proposal_id.cmp(&right.proposal_id))
            .then_with(|| left.subnet_id.cmp(&right.subnet_id))
    });
    Ok(())
}

#[cfg(test)]
mod tests;
