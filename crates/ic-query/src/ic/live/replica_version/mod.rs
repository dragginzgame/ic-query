//! Module: ic::live::replica_version
//!
//! Responsibility: live Dashboard replica-version URL and wire handling.
//! Does not own: report validation, CLI parsing, provenance projection, or rendering.
//! Boundary: performs one bounded list request or one exact release request.

use super::{LiveIcSource, dashboard_resource_url};
use crate::ic::{
    IcHostError, IcReplicaVersionInfoSourceData, IcReplicaVersionListQuery,
    IcReplicaVersionListRow, IcReplicaVersionListSourceData, IcReplicaVersionSource,
    IcReplicaVersionStatus, IcReplicaVersionSubnetRollout, IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcReplicaVersionSource for LiveIcSource {
    fn fetch_replica_version_list<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcReplicaVersionListQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcReplicaVersionListSourceData, IcHostError>> {
        Box::pin(async move {
            source::validate_replica_version_list_query(query)?;
            let url = replica_version_list_url(&request.endpoint, query)?;
            let wire: DashboardReplicaVersionList = self.fetch_live(url).await?;
            wire.into_source_data(request, query)
        })
    }

    fn fetch_replica_version_info<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        replica_version_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<IcReplicaVersionInfoSourceData, IcHostError>> {
        Box::pin(async move {
            source::validate_replica_version_id(replica_version_id)?;
            let url = replica_version_info_url(&request.endpoint, replica_version_id)?;
            let wire: crate::ic::IcReplicaVersionDetails = self.fetch_live(url).await?;
            Ok(IcReplicaVersionInfoSourceData {
                source: request.clone(),
                release: wire,
            })
        })
    }
}

fn replica_version_list_url(
    endpoint: &str,
    query: &IcReplicaVersionListQuery,
) -> Result<Url, IcHostError> {
    let mut url = dashboard_resource_url(endpoint, &["subnet-replica-versions"])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("limit", &query.limit.to_string());
        pairs.append_pair("offset", &query.offset.to_string());
        pairs.append_pair("sort_by", "-executed_timestamp_seconds");
        for status in [
            IcReplicaVersionStatus::Executed,
            IcReplicaVersionStatus::Open,
            IcReplicaVersionStatus::Adopted,
        ] {
            pairs.append_pair("include_status", status.as_dashboard_value());
        }
        if let Some(max_proposal_index) = query.max_proposal_index {
            pairs.append_pair("max_proposal_index", &max_proposal_index.to_string());
        }
    }
    Ok(url)
}

fn replica_version_info_url(endpoint: &str, replica_version_id: &str) -> Result<Url, IcHostError> {
    dashboard_resource_url(endpoint, &["subnet-replica-versions", replica_version_id])
}

#[derive(SerdeDeserialize)]
struct DashboardReplicaVersionList {
    data: Vec<DashboardReplicaVersionListRow>,
    max_proposal_index: u64,
    total_proposals: u64,
}

impl DashboardReplicaVersionList {
    fn into_source_data(
        self,
        request: &IcSourceRequest,
        query: &IcReplicaVersionListQuery,
    ) -> Result<IcReplicaVersionListSourceData, IcHostError> {
        Ok(IcReplicaVersionListSourceData {
            source: request.clone(),
            query: query.clone(),
            resolved_max_proposal_index: self.max_proposal_index,
            total_proposals: self.total_proposals,
            rows: self
                .data
                .into_iter()
                .map(DashboardReplicaVersionListRow::into_public)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(SerdeDeserialize)]
struct DashboardReplicaVersionListRow {
    executed_timestamp_seconds: u64,
    proposal_id: u64,
    replica_version_id: String,
    status: String,
    subnets: Vec<IcReplicaVersionSubnetRollout>,
    title: String,
    url: String,
}

impl DashboardReplicaVersionListRow {
    fn into_public(self) -> Result<IcReplicaVersionListRow, IcHostError> {
        Ok(IcReplicaVersionListRow {
            replica_version_id: self.replica_version_id,
            proposal_id: self.proposal_id,
            executed_timestamp_seconds: self.executed_timestamp_seconds,
            status: replica_version_status(&self.status)?,
            title: self.title,
            url: self.url,
            subnets: self.subnets,
        })
    }
}

fn replica_version_status(value: &str) -> Result<IcReplicaVersionStatus, IcHostError> {
    match value {
        "ADOPTED" => Ok(IcReplicaVersionStatus::Adopted),
        "EXECUTED" => Ok(IcReplicaVersionStatus::Executed),
        "OPEN" => Ok(IcReplicaVersionStatus::Open),
        _ => Err(IcHostError::InvalidSourceData {
            reason: format!("unknown replica-version status {value:?}"),
        }),
    }
}

#[cfg(test)]
mod tests;
