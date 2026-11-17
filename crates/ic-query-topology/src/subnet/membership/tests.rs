use std::cell::Cell;
use std::future::{pending, ready};
use std::time::Duration;

use super::*;
use crate::ReadError;
use crate::subnet::SubnetEvidence;
use crate::subnet::configuration::SubnetConfigurationEvidence;

fn id(byte: u8) -> Principal {
    Principal::from_slice(&[byte])
}

fn configuration(nodes: Vec<Principal>) -> SubnetConfigurationReport {
    SubnetConfigurationReport {
        schema: 1,
        subnet_id: id(10),
        registered_node_ids: nodes,
        replica_version_id: "configured".into(),
        registered_subnet_type: 2,
        is_halted: false,
        halt_at_cup_height: true,
        evidence: SubnetConfigurationEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "get_value",
            source_key: "subnet_record_fixture".into(),
            observed_at_unix_seconds: 123,
            record_version: 456,
            record_timestamp_nanoseconds: u64::MAX,
            query_signature_verified: true,
            result_certified: false,
            membership_complete: true,
        },
    }
}

fn observation(nodes: Vec<Principal>) -> SubnetReport {
    SubnetReport {
        schema: 1,
        subnet_id: id(10),
        observed_node_ids: nodes,
        observed_subnet_type: Some("system".into()),
        evidence: SubnetEvidence {
            endpoint: "fixture",
            source_path: "/read_state".into(),
            observed_at_unix_seconds: 122,
            state_certificate_verified: true,
            membership_complete: false,
            registry_version: None,
        },
    }
}

#[tokio::test]
async fn differences_preserve_source_evidence_without_promoting_assurance() {
    for (registered, observed, registered_only, observed_only) in [
        (vec![1, 2], vec![1, 2], vec![], vec![]),
        (vec![1, 3], vec![2, 4], vec![1, 3], vec![2, 4]),
        (vec![1, 2], vec![2, 3], vec![1], vec![3]),
        (vec![], vec![1], vec![], vec![1]),
    ] {
        let report = collect(
            id(10),
            ready(Ok(configuration(registered.into_iter().map(id).collect()))),
            ready(Ok(observation(observed.into_iter().map(id).collect()))),
        )
        .await
        .unwrap();
        assert_eq!(
            report.registered_without_observed_key,
            registered_only.into_iter().map(id).collect::<Vec<_>>()
        );
        assert_eq!(
            report.observed_key_without_registration,
            observed_only.into_iter().map(id).collect::<Vec<_>>()
        );
        let json = serde_json::to_value(report).unwrap();
        assert_eq!(json["atomic_snapshot"], false);
        assert_eq!(json["configuration"]["evidence"]["record_version"], 456);
        assert_eq!(
            json["configuration"]["evidence"]["record_timestamp_nanoseconds"],
            u64::MAX
        );
        assert_eq!(json["configuration"]["evidence"]["result_certified"], false);
        assert_eq!(
            json["configuration"]["evidence"]["membership_complete"],
            true
        );
        assert_eq!(
            json["observation"]["evidence"]["state_certificate_verified"],
            true
        );
        assert_eq!(
            json["observation"]["evidence"]["membership_complete"],
            false
        );
        assert_eq!(
            json["configuration"]["evidence"]["observed_at_unix_seconds"],
            123
        );
        assert_eq!(
            json["observation"]["evidence"]["observed_at_unix_seconds"],
            122
        );
        assert!(json["observation"]["evidence"]["registry_version"].is_null());
    }
}

#[tokio::test]
async fn differences_retain_each_sources_exact_member_limit() {
    let registered: Vec<_> = (0_u64..1024)
        .map(|n| Principal::from_slice(&n.to_be_bytes()))
        .collect();
    let observed: Vec<_> = (1024_u64..2048)
        .map(|n| Principal::from_slice(&n.to_be_bytes()))
        .collect();
    let report = collect(
        id(10),
        ready(Ok(configuration(registered.clone()))),
        ready(Ok(observation(observed.clone()))),
    )
    .await
    .unwrap();
    assert_eq!(report.registered_without_observed_key, registered);
    assert_eq!(report.observed_key_without_registration, observed);
}

#[tokio::test]
async fn either_wrong_subnet_is_refused() {
    for first_wrong in [true, false] {
        let mut configured = configuration(vec![id(1)]);
        let mut observed = observation(vec![id(1)]);
        if first_wrong {
            configured.subnet_id = id(11);
        } else {
            observed.subnet_id = id(11);
        }
        assert!(
            matches!(collect(id(10), ready(Ok(configured)), ready(Ok(observed))).await,
            Err(SubnetMembershipError::IdentityMismatch { requested, .. }) if requested == id(10))
        );
    }
}

#[tokio::test]
async fn first_failure_never_polls_the_second_reader() {
    let polled = Cell::new(false);
    let result = collect(
        id(10),
        ready(Err(SubnetConfigurationError::MissingContent)),
        async {
            polled.set(true);
            Ok(observation(vec![id(1)]))
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(SubnetMembershipError::Configuration(
            SubnetConfigurationError::MissingContent
        ))
    ));
    assert!(!polled.get());
}

#[tokio::test]
async fn second_failure_follows_configuration_and_retains_it() {
    let first_done = Cell::new(false);
    let result = collect(
        id(10),
        async {
            first_done.set(true);
            Ok(configuration(vec![id(1)]))
        },
        async {
            assert!(first_done.get());
            Err(SubnetError::Read(ReadError::Deadline))
        },
    )
    .await;
    match result {
        Err(SubnetMembershipError::Observation {
            configuration,
            source,
        }) => {
            assert_eq!(configuration.registered_node_ids, [id(1)]);
            assert_eq!(configuration.evidence.record_version, 456);
            assert!(matches!(source, SubnetError::Read(ReadError::Deadline)));
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn cancellation_during_first_read_never_starts_the_second() {
    let polled = Cell::new(false);
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        collect(id(10), pending(), async {
            polled.set(true);
            Ok(observation(vec![id(1)]))
        }),
    )
    .await;
    assert!(result.is_err());
    tokio::time::advance(Duration::from_secs(60)).await;
    assert!(!polled.get());
}
