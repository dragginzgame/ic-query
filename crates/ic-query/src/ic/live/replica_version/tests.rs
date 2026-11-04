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
        format!("https://ic-api.internetcomputer.org/api/v3/subnet-replica-versions/{VERSION_ID}")
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
                    "subnets": [{{
                        "subnet_id": "aaaaa-aa",
                        "proposal_id": 143400,
                        "executed_timestamp_seconds": 100,
                        "future_field": "ignored"
                    }}],
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
    assert_eq!(source.rows[0].subnets.len(), 1);
    assert_eq!(
        source.rows[0].subnets[0],
        IcReplicaVersionSubnetRollout {
            subnet_id: "aaaaa-aa".to_string(),
            proposal_id: 143_400,
            executed_timestamp_seconds: 100,
        }
    );
}

#[test]
fn unknown_list_status_is_rejected() {
    assert!(matches!(
        replica_version_status("FAILED"),
        Err(IcHostError::InvalidSourceData { .. })
    ));
}
