use super::*;
use crate::ic::DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT;
use crate::ic::IcReplicaVersionInfoRequest;
use crate::ic::IcReplicaVersionListRequest;
use crate::ic::build_ic_replica_version_info_report_with_source;
use crate::ic::build_ic_replica_version_list_report_with_source;
use std::sync::Mutex;

const VERSION_A: &str = "e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3";
const VERSION_B: &str = "0f974949344626daf5d28f578a0de26c5734e580";
const SUBNET_A: &str = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";
const SUBNET_B: &str = "2fq7c-slacv-26cgz-vzbx2-2jrcs-5edph-i5s2j-tck77-c3rlz-iobzx-mqe";

#[tokio::test(flavor = "current_thread")]
async fn bounded_list_preserves_metadata_status_rows_and_provenance() {
    let source = FixtureSource::default();
    let request = list_request();
    let report = build_ic_replica_version_list_report_with_source(&request, &source)
        .await
        .expect("replica-version list");

    assert_eq!((*source.list_calls.lock().unwrap()), 1);
    assert_eq!((*source.info_calls.lock().unwrap()), 0);
    assert_eq!(report.rows.len(), 2);
    assert_eq!(report.total_proposals, 3);
    assert_eq!(report.resolved_max_proposal_index, 438);
    assert_eq!(report.next_offset, Some(2));
    assert_eq!(report.rows[0].status, IcReplicaVersionStatus::Executed);
    assert_eq!(report.rows[1].status, IcReplicaVersionStatus::Open);
    assert_eq!(report.rows[1].executed_timestamp_seconds, 0);
    assert_eq!(report.rows[1].replica_version_id, VERSION_A);
    assert_eq!(report.rows[1].title, "");
    assert_eq!(report.rows[1].url, "");
    assert_eq!(report.provenance.authority, "official_ic_dashboard_api");
    assert!(!report.provenance.certified);
    assert!(!report.provenance.point_in_time_guaranteed);
}

#[tokio::test(flavor = "current_thread")]
async fn exact_info_preserves_summary_and_canonically_orders_rollouts() {
    let source = FixtureSource::default();
    let request = info_request();
    let report = build_ic_replica_version_info_report_with_source(&request, &source)
        .await
        .expect("replica-version info");

    assert_eq!((*source.info_calls.lock().unwrap()), 1);
    assert_eq!((*source.list_calls.lock().unwrap()), 0);
    assert_eq!(report.release.replica_version_id, VERSION_A);
    assert_eq!(report.release.summary, "Release notes\nwith raw detail");
    assert_eq!(report.release.subnets.len(), 2);
    assert_eq!(report.release.subnets[0].subnet_id, SUBNET_A);
    assert_eq!(report.release.subnets[1].subnet_id, SUBNET_B);
}

#[test]
fn unexecuted_subnet_rollout_preserves_zero_timestamp() {
    let mut rollouts = vec![rollout(SUBNET_A, 143_297, 0)];

    validate_subnet_rollouts(&mut rollouts).expect("unexecuted rollout");

    assert_eq!(rollouts[0].executed_timestamp_seconds, 0);
}

#[tokio::test(flavor = "current_thread")]
async fn request_validation_precedes_custom_source_calls() {
    let invalid_queries = [
        IcReplicaVersionListQuery::new(0, 0, None),
        IcReplicaVersionListQuery::new(MAX_IC_REPLICA_VERSION_PAGE_LIMIT + 1, 0, None),
    ];
    for query in invalid_queries {
        let source = FixtureSource::default();
        let request = IcReplicaVersionListRequest::new(
            crate::ic::IcDashboardRequest::new(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, 1_800_000_000),
            query,
        );
        let error = build_ic_replica_version_list_report_with_source(&request, &source)
            .await
            .expect_err("invalid query must fail");
        assert!(matches!(error, IcHostError::InvalidRequest { .. }));
        assert_eq!((*source.list_calls.lock().unwrap()), 0);
    }

    let source = FixtureSource::default();
    let request = IcReplicaVersionInfoRequest::new(
        crate::ic::IcDashboardRequest::new(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, 1_800_000_000),
        "not-a-version",
    );
    let error = build_ic_replica_version_info_report_with_source(&request, &source)
        .await
        .expect_err("invalid version id must fail");
    assert!(matches!(error, IcHostError::InvalidRequest { .. }));
    assert_eq!((*source.info_calls.lock().unwrap()), 0);
}

#[tokio::test(flavor = "current_thread")]
async fn custom_source_identity_order_and_metadata_are_validated() {
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
            mutation: Mutex::new(Some(mutation)),
            ..FixtureSource::default()
        };
        let error = if mutation == Mutation::WrongInfoVersion {
            build_ic_replica_version_info_report_with_source(&info_request(), &source)
                .await
                .expect_err("invalid info source must fail")
        } else {
            let mut request = list_request();
            if mutation == Mutation::MaximumAboveRequest {
                request.query.max_proposal_index = Some(438);
            }
            build_ic_replica_version_list_report_with_source(&request, &source)
                .await
                .expect_err("invalid list source must fail")
        };

        assert!(matches!(error, IcHostError::InvalidSourceData { .. }));
    }
}

fn list_request() -> IcReplicaVersionListRequest {
    IcReplicaVersionListRequest::new(
        crate::ic::IcDashboardRequest::new(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, 1_800_000_000),
        IcReplicaVersionListQuery::new(2, 0, None),
    )
}

fn info_request() -> IcReplicaVersionInfoRequest {
    IcReplicaVersionInfoRequest::new(
        crate::ic::IcDashboardRequest::new(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, 1_800_000_000),
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
    list_calls: Mutex<usize>,
    info_calls: Mutex<usize>,
    mutation: Mutex<Option<Mutation>>,
}

impl IcReplicaVersionSource for FixtureSource {
    fn fetch_replica_version_list<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcReplicaVersionListQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcReplicaVersionListSourceData, IcHostError>> {
        Box::pin(async move {
            *self.list_calls.lock().unwrap() += 1;
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
            let mutation = self.mutation.lock().unwrap().take();
            match mutation {
                Some(Mutation::WrongQuery) => source.query.limit = 0,
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
        })
    }

    fn fetch_replica_version_info<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        replica_version_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<IcReplicaVersionInfoSourceData, IcHostError>> {
        Box::pin(async move {
            *self.info_calls.lock().unwrap() += 1;
            let mut source = IcReplicaVersionInfoSourceData {
                release: crate::ic::IcReplicaVersionDetails {
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
                },
                source: request.clone(),
            };
            if self.mutation.lock().unwrap().take() == Some(Mutation::WrongInfoVersion) {
                source.release.replica_version_id = VERSION_B.to_string();
            }
            Ok(source)
        })
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
