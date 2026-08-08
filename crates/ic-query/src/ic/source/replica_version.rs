//! Module: ic::source::replica_version
//!
//! Responsibility: Dashboard replica-version source contract, validation, and projection.
//! Does not own: HTTP transport, shared provenance, CLI paging, or text rendering.
//! Boundary: validates one bounded page or one exact release record without runtime claims.

use super::{
    invalid_request, invalid_source, report_provenance, validate_canonical_principal,
    validate_provenance,
};
use crate::{
    hex::is_lowercase_hex,
    ic::{
        IcHostError, IcReplicaVersionInfoReport, IcReplicaVersionInfoSourceData,
        IcReplicaVersionListQuery, IcReplicaVersionListReport, IcReplicaVersionListRow,
        IcReplicaVersionListSourceData, IcReplicaVersionStatus, IcReplicaVersionSubnetRollout,
        IcSourceRequest, MAX_IC_REPLICA_VERSION_PAGE_LIMIT,
    },
};
use std::collections::HashSet;

///
/// IcReplicaVersionSource
///
/// Source contract for bounded official Dashboard replica-version reports.
///

pub trait IcReplicaVersionSource {
    /// Fetch one explicitly bounded release page without automatically paginating.
    fn fetch_replica_version_list(
        &self,
        request: &IcSourceRequest,
        query: &IcReplicaVersionListQuery,
    ) -> Result<IcReplicaVersionListSourceData, IcHostError>;

    /// Fetch one exact release record by replica-version identifier.
    fn fetch_replica_version_info(
        &self,
        request: &IcSourceRequest,
        replica_version_id: &str,
    ) -> Result<IcReplicaVersionInfoSourceData, IcHostError>;
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
    if source.query != *query {
        return invalid_source(format!(
            "replica-version query is {:?}, expected requested query {query:?}",
            source.query
        ));
    }
    validate_replica_version_list_query(&source.query)?;
    if let Some(requested_maximum) = source.query.max_proposal_index
        && source.resolved_max_proposal_index > requested_maximum
    {
        return invalid_source(format!(
            "resolved_max_proposal_index is {}, greater than requested ceiling {requested_maximum}",
            source.resolved_max_proposal_index
        ));
    }
    if source.rows.len() > usize::from(source.query.limit) {
        return invalid_source(format!(
            "source returned {} replica-version rows for requested limit {}",
            source.rows.len(),
            source.query.limit
        ));
    }
    validate_list_metadata(&source.query, source.total_proposals, source.rows.len())?;
    validate_list_rows(&mut source.rows)?;

    let returned_count = source.rows.len();
    let consumed = source
        .query
        .offset
        .checked_add(u64::try_from(returned_count).unwrap_or(u64::MAX))
        .ok_or_else(|| IcHostError::InvalidSourceData {
            reason: "replica-version page offset overflows u64".to_string(),
        })?;
    let next_offset = (returned_count > 0 && consumed < source.total_proposals).then_some(consumed);

    Ok(IcReplicaVersionListReport {
        provenance: report_provenance(source.source),
        query: source.query,
        resolved_max_proposal_index: source.resolved_max_proposal_index,
        total_proposals: source.total_proposals,
        returned_count,
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
        &source.replica_version_id,
        source.proposal_id,
    )?;
    if source.replica_version_id != requested_replica_version_id {
        return invalid_source(format!(
            "replica_version_id is {:?}, expected requested value {requested_replica_version_id:?}",
            source.replica_version_id
        ));
    }
    validate_subnet_rollouts(&mut source.subnets)?;

    Ok(IcReplicaVersionInfoReport {
        provenance: report_provenance(source.source),
        replica_version_id: source.replica_version_id,
        proposal_id: source.proposal_id,
        executed_timestamp_seconds: source.executed_timestamp_seconds,
        title: source.title,
        url: source.url,
        summary: source.summary,
        subnet_count: source.subnets.len(),
        subnets: source.subnets,
    })
}

fn validate_list_metadata(
    query: &IcReplicaVersionListQuery,
    total_proposals: u64,
    returned_count: usize,
) -> Result<(), IcHostError> {
    let returned_count = u64::try_from(returned_count).unwrap_or(u64::MAX);
    if returned_count > total_proposals {
        return invalid_source(format!(
            "returned_count {returned_count} exceeds total_proposals {total_proposals}"
        ));
    }
    if returned_count == 0 {
        return Ok(());
    }
    if query.offset >= total_proposals {
        return invalid_source(format!(
            "nonempty page starts at offset {}, but total_proposals is {total_proposals}",
            query.offset
        ));
    }
    let consumed =
        query
            .offset
            .checked_add(returned_count)
            .ok_or_else(|| IcHostError::InvalidSourceData {
                reason: "replica-version page offset overflows u64".to_string(),
            })?;
    if consumed > total_proposals {
        return invalid_source(format!(
            "page ending at offset {consumed} exceeds total_proposals {total_proposals}"
        ));
    }
    Ok(())
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
        if row.subnet_count != row.subnets.len() {
            return invalid_source(format!(
                "row.subnet_count is {}, expected {} from returned Subnet rows",
                row.subnet_count,
                row.subnets.len()
            ));
        }
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
mod tests {
    use super::*;
    use crate::ic::{
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcReplicaVersionInfoRequest,
        IcReplicaVersionListRequest, build_ic_replica_version_info_report_with_source,
        build_ic_replica_version_list_report_with_source, ic_replica_version_info_report_text,
        ic_replica_version_list_report_text,
    };
    use std::cell::Cell;

    const VERSION_A: &str = "e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3";
    const VERSION_B: &str = "0f974949344626daf5d28f578a0de26c5734e580";
    const SUBNET_A: &str = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";
    const SUBNET_B: &str = "2fq7c-slacv-26cgz-vzbx2-2jrcs-5edph-i5s2j-tck77-c3rlz-iobzx-mqe";

    #[test]
    fn bounded_list_preserves_metadata_status_rows_and_provenance() {
        let source = FixtureSource::default();
        let request = list_request();
        let report = build_ic_replica_version_list_report_with_source(&request, &source)
            .expect("replica-version list");
        let text = ic_replica_version_list_report_text(&report);

        assert_eq!(source.list_calls.get(), 1);
        assert_eq!(source.info_calls.get(), 0);
        assert_eq!(report.returned_count, 2);
        assert_eq!(report.total_proposals, 3);
        assert_eq!(report.resolved_max_proposal_index, 438);
        assert_eq!(report.next_offset, Some(2));
        assert_eq!(report.rows[0].status, IcReplicaVersionStatus::Executed);
        assert_eq!(report.rows[1].status, IcReplicaVersionStatus::Open);
        assert_eq!(report.rows[1].executed_timestamp_seconds, 0);
        assert_eq!(report.rows[1].replica_version_id, VERSION_A);
        assert!(report.rows[1].title.is_empty());
        assert!(report.rows[1].url.is_empty());
        assert_eq!(report.provenance.authority, "official_ic_dashboard_api");
        assert!(!report.provenance.certified);
        assert!(!report.provenance.point_in_time_guaranteed);
        assert!(text.contains("resolved_max_proposal_index: 438"));
        assert!(text.contains("EXECUTED"));
        assert!(text.contains("OPEN"));
    }

    #[test]
    fn exact_info_preserves_summary_and_canonically_orders_rollouts() {
        let source = FixtureSource::default();
        let request = info_request();
        let report = build_ic_replica_version_info_report_with_source(&request, &source)
            .expect("replica-version info");
        let text = ic_replica_version_info_report_text(&report);

        assert_eq!(source.info_calls.get(), 1);
        assert_eq!(source.list_calls.get(), 0);
        assert_eq!(report.replica_version_id, VERSION_A);
        assert_eq!(report.summary, "Release notes\nwith raw detail");
        assert_eq!(report.subnet_count, 2);
        assert_eq!(report.subnets[0].subnet_id, SUBNET_A);
        assert_eq!(report.subnets[1].subnet_id, SUBNET_B);
        assert!(text.contains("summary: Release notes\\nwith raw detail"));
        assert!(text.contains("subnet_rollouts:"));
    }

    #[test]
    fn unexecuted_subnet_rollout_preserves_zero_timestamp() {
        let mut rollouts = vec![rollout(SUBNET_A, 143_297, 0)];

        validate_subnet_rollouts(&mut rollouts).expect("unexecuted rollout");

        assert_eq!(rollouts[0].executed_timestamp_seconds, 0);
    }

    #[test]
    fn request_validation_precedes_custom_source_calls() {
        let invalid_queries = [
            IcReplicaVersionListQuery::new(0, 0, None),
            IcReplicaVersionListQuery::new(MAX_IC_REPLICA_VERSION_PAGE_LIMIT + 1, 0, None),
        ];
        for query in invalid_queries {
            let source = FixtureSource::default();
            let request = IcReplicaVersionListRequest::new(
                DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
                1_800_000_000,
                query,
            );
            let error = build_ic_replica_version_list_report_with_source(&request, &source)
                .expect_err("invalid query must fail");
            assert!(matches!(error, IcHostError::InvalidRequest { .. }));
            assert_eq!(source.list_calls.get(), 0);
        }

        let source = FixtureSource::default();
        let request = IcReplicaVersionInfoRequest::new(
            DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
            1_800_000_000,
            "not-a-version",
        );
        let error = build_ic_replica_version_info_report_with_source(&request, &source)
            .expect_err("invalid version id must fail");
        assert!(matches!(error, IcHostError::InvalidRequest { .. }));
        assert_eq!(source.info_calls.get(), 0);
    }

    #[test]
    fn custom_source_identity_order_and_metadata_are_validated() {
        for mutation in [
            Mutation::WrongQuery,
            Mutation::MaximumAboveRequest,
            Mutation::TooManyRows,
            Mutation::DuplicateProposal,
            Mutation::UnorderedRows,
            Mutation::ExecutedWithoutTimestamp,
            Mutation::WrongInfoVersion,
            Mutation::InvalidSubnet,
        ] {
            let source = FixtureSource {
                mutation: Cell::new(Some(mutation)),
                ..FixtureSource::default()
            };
            let error = if mutation == Mutation::WrongInfoVersion {
                build_ic_replica_version_info_report_with_source(&info_request(), &source)
                    .expect_err("invalid info source must fail")
            } else {
                let mut request = list_request();
                if mutation == Mutation::MaximumAboveRequest {
                    request.query.max_proposal_index = Some(438);
                }
                build_ic_replica_version_list_report_with_source(&request, &source)
                    .expect_err("invalid list source must fail")
            };

            assert!(matches!(error, IcHostError::InvalidSourceData { .. }));
        }
    }

    fn list_request() -> IcReplicaVersionListRequest {
        IcReplicaVersionListRequest::new(
            DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
            1_800_000_000,
            IcReplicaVersionListQuery::new(2, 0, None),
        )
    }

    fn info_request() -> IcReplicaVersionInfoRequest {
        IcReplicaVersionInfoRequest::new(
            DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
            1_800_000_000,
            VERSION_A,
        )
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Mutation {
        WrongQuery,
        MaximumAboveRequest,
        TooManyRows,
        DuplicateProposal,
        UnorderedRows,
        ExecutedWithoutTimestamp,
        WrongInfoVersion,
        InvalidSubnet,
    }

    #[derive(Default)]
    struct FixtureSource {
        list_calls: Cell<usize>,
        info_calls: Cell<usize>,
        mutation: Cell<Option<Mutation>>,
    }

    impl IcReplicaVersionSource for FixtureSource {
        fn fetch_replica_version_list(
            &self,
            request: &IcSourceRequest,
            query: &IcReplicaVersionListQuery,
        ) -> Result<IcReplicaVersionListSourceData, IcHostError> {
            self.list_calls.set(self.list_calls.get() + 1);
            let mut source = IcReplicaVersionListSourceData {
                source: request.clone(),
                query: query.clone(),
                resolved_max_proposal_index: 438,
                total_proposals: 3,
                rows: vec![
                    list_row(
                        VERSION_A,
                        143_250,
                        1_785_759_673,
                        IcReplicaVersionStatus::Executed,
                    ),
                    list_row(VERSION_A, 143_249, 0, IcReplicaVersionStatus::Open),
                ],
            };
            source.rows[1].title.clear();
            source.rows[1].url.clear();
            match self.mutation.take() {
                Some(Mutation::WrongQuery) => source.query.offset += 1,
                Some(Mutation::MaximumAboveRequest) => {
                    source.resolved_max_proposal_index = 439;
                }
                Some(Mutation::TooManyRows) => {
                    source.rows.push(list_row(
                        "1153c70d98a51ec2f023be9b3ce1d50c6b67da21",
                        143_248,
                        0,
                        IcReplicaVersionStatus::Open,
                    ));
                    source.total_proposals = 4;
                }
                Some(Mutation::DuplicateProposal) => {
                    source.rows[1].proposal_id = source.rows[0].proposal_id;
                }
                Some(Mutation::UnorderedRows) => {
                    source.rows[1].executed_timestamp_seconds = 1_785_759_674;
                }
                Some(Mutation::ExecutedWithoutTimestamp) => {
                    source.rows[0].executed_timestamp_seconds = 0;
                }
                Some(Mutation::InvalidSubnet) => {
                    source.rows[0].subnets[0].subnet_id = "not a principal".to_string();
                }
                Some(Mutation::WrongInfoVersion) | None => {}
            }
            Ok(source)
        }

        fn fetch_replica_version_info(
            &self,
            request: &IcSourceRequest,
            replica_version_id: &str,
        ) -> Result<IcReplicaVersionInfoSourceData, IcHostError> {
            self.info_calls.set(self.info_calls.get() + 1);
            let mut source = IcReplicaVersionInfoSourceData {
                source: request.clone(),
                replica_version_id: replica_version_id.to_string(),
                proposal_id: 143_250,
                executed_timestamp_seconds: 1_785_759_673,
                title: "Elect release".to_string(),
                url: "https://forum.dfinity.org/t/release/1".to_string(),
                summary: "Release notes\nwith raw detail".to_string(),
                subnets: vec![
                    rollout(SUBNET_B, 143_298, 1_785_759_900),
                    rollout(SUBNET_A, 143_297, 1_785_759_892),
                ],
            };
            if self.mutation.take() == Some(Mutation::WrongInfoVersion) {
                source.replica_version_id = VERSION_B.to_string();
            }
            Ok(source)
        }
    }

    fn list_row(
        replica_version_id: &str,
        proposal_id: u64,
        executed_timestamp_seconds: u64,
        status: IcReplicaVersionStatus,
    ) -> IcReplicaVersionListRow {
        let subnets = if status == IcReplicaVersionStatus::Executed {
            vec![rollout(SUBNET_A, 143_297, 1_785_759_892)]
        } else {
            Vec::new()
        };
        IcReplicaVersionListRow {
            replica_version_id: replica_version_id.to_string(),
            proposal_id,
            executed_timestamp_seconds,
            status,
            title: "Elect release".to_string(),
            url: "https://forum.dfinity.org/t/release/1".to_string(),
            subnet_count: subnets.len(),
            subnets,
        }
    }

    fn rollout(
        subnet_id: &str,
        proposal_id: u64,
        executed_timestamp_seconds: u64,
    ) -> IcReplicaVersionSubnetRollout {
        IcReplicaVersionSubnetRollout {
            subnet_id: subnet_id.to_string(),
            proposal_id,
            executed_timestamp_seconds,
        }
    }
}
