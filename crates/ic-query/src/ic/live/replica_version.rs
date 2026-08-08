//! Module: ic::live::replica_version
//!
//! Responsibility: live Dashboard replica-version URL and wire handling.
//! Does not own: report validation, CLI parsing, provenance projection, or rendering.
//! Boundary: performs one bounded list request or one exact release request.

use super::{LiveIcSource, append_path_segments, dashboard_base_url, fetch_live};
use crate::ic::{
    IcHostError, IcReplicaVersionInfoSourceData, IcReplicaVersionListQuery,
    IcReplicaVersionListRow, IcReplicaVersionListSourceData, IcReplicaVersionSource,
    IcReplicaVersionStatus, IcReplicaVersionSubnetRollout, IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcReplicaVersionSource for LiveIcSource {
    fn fetch_replica_version_list(
        &self,
        request: &IcSourceRequest,
        query: &IcReplicaVersionListQuery,
    ) -> Result<IcReplicaVersionListSourceData, IcHostError> {
        source::validate_replica_version_list_query(query)?;
        let url = replica_version_list_url(&request.endpoint, query)?;
        let wire: DashboardReplicaVersionList = fetch_live(url)?;
        wire.into_source_data(request, query)
    }

    fn fetch_replica_version_info(
        &self,
        request: &IcSourceRequest,
        replica_version_id: &str,
    ) -> Result<IcReplicaVersionInfoSourceData, IcHostError> {
        source::validate_replica_version_id(replica_version_id)?;
        let url = replica_version_info_url(&request.endpoint, replica_version_id)?;
        let wire: DashboardReplicaVersionInfo = fetch_live(url)?;
        Ok(wire.into_source_data(request))
    }
}

fn replica_version_list_url(
    endpoint: &str,
    query: &IcReplicaVersionListQuery,
) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["subnet-replica-versions"])?;
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
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(
        endpoint,
        &mut url,
        &["subnet-replica-versions", replica_version_id],
    )?;
    Ok(url)
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
    subnets: Vec<DashboardReplicaVersionSubnet>,
    title: String,
    url: String,
}

impl DashboardReplicaVersionListRow {
    fn into_public(self) -> Result<IcReplicaVersionListRow, IcHostError> {
        let subnet_count = self.subnets.len();
        Ok(IcReplicaVersionListRow {
            replica_version_id: self.replica_version_id,
            proposal_id: self.proposal_id,
            executed_timestamp_seconds: self.executed_timestamp_seconds,
            status: replica_version_status(&self.status)?,
            title: self.title,
            url: self.url,
            subnet_count,
            subnets: self
                .subnets
                .into_iter()
                .map(DashboardReplicaVersionSubnet::into_public)
                .collect(),
        })
    }
}

#[derive(SerdeDeserialize)]
struct DashboardReplicaVersionInfo {
    executed_timestamp_seconds: u64,
    proposal_id: u64,
    replica_version_id: String,
    subnets: Vec<DashboardReplicaVersionSubnet>,
    summary: String,
    title: String,
    url: String,
}

impl DashboardReplicaVersionInfo {
    fn into_source_data(self, request: &IcSourceRequest) -> IcReplicaVersionInfoSourceData {
        IcReplicaVersionInfoSourceData {
            source: request.clone(),
            replica_version_id: self.replica_version_id,
            proposal_id: self.proposal_id,
            executed_timestamp_seconds: self.executed_timestamp_seconds,
            title: self.title,
            url: self.url,
            summary: self.summary,
            subnets: self
                .subnets
                .into_iter()
                .map(DashboardReplicaVersionSubnet::into_public)
                .collect(),
        }
    }
}

#[derive(SerdeDeserialize)]
struct DashboardReplicaVersionSubnet {
    executed_timestamp_seconds: u64,
    proposal_id: u64,
    subnet_id: String,
}

impl DashboardReplicaVersionSubnet {
    fn into_public(self) -> IcReplicaVersionSubnetRollout {
        IcReplicaVersionSubnetRollout {
            subnet_id: self.subnet_id,
            proposal_id: self.proposal_id,
            executed_timestamp_seconds: self.executed_timestamp_seconds,
        }
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
mod tests {
    use super::*;

    const VERSION_ID: &str = "e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3";

    #[test]
    fn replica_version_urls_preserve_explicit_bounds_and_exact_target() {
        let query = IcReplicaVersionListQuery::new(25, 50, Some(438));
        assert_eq!(
            replica_version_list_url("https://ic-api.internetcomputer.org/api/v3/", &query,)
                .expect("list URL")
                .as_str(),
            "https://ic-api.internetcomputer.org/api/v3/subnet-replica-versions?limit=25&offset=50&sort_by=-executed_timestamp_seconds&include_status=EXECUTED&include_status=OPEN&include_status=ADOPTED&max_proposal_index=438"
        );
        assert_eq!(
            replica_version_info_url("https://ic-api.internetcomputer.org/api/v3", VERSION_ID,)
                .expect("info URL")
                .as_str(),
            format!(
                "https://ic-api.internetcomputer.org/api/v3/subnet-replica-versions/{VERSION_ID}"
            )
        );
    }

    #[test]
    fn list_wire_preserves_open_status_and_discards_only_list_summary() {
        let wire: DashboardReplicaVersionList = serde_json::from_str(&format!(
            r#"{{
                "data": [{{
                    "executed_timestamp_seconds": 0,
                    "proposal_id": 143406,
                    "replica_version_id": "{VERSION_ID}",
                    "status": "OPEN",
                    "subnets": [],
                    "summary": "large detail is available through info",
                    "title": "Elect release",
                    "url": "https://forum.dfinity.org/t/release/1"
                }}],
                "max_proposal_index": 2,
                "total_proposals": 2
            }}"#
        ))
        .expect("list wire");
        let source = wire
            .into_source_data(
                &IcSourceRequest::new("https://example.com", "2026-08-08T00:00:00Z", "test"),
                &IcReplicaVersionListQuery::new(1, 0, None),
            )
            .expect("source data");

        assert_eq!(source.rows[0].status, IcReplicaVersionStatus::Open);
        assert_eq!(source.rows[0].executed_timestamp_seconds, 0);
        assert_eq!(source.resolved_max_proposal_index, 2);
    }

    #[test]
    fn unknown_list_status_is_rejected() {
        assert!(matches!(
            replica_version_status("FAILED"),
            Err(IcHostError::InvalidSourceData { .. })
        ));
    }
}
