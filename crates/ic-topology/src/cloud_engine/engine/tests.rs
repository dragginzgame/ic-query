use super::*;
use crate::ReadError;
use ic_platform::candid_codec::encode_one;
use std::{cell::Cell, time::Duration};

// Independently assembled from the selected IDL and Candid binary specification,
// not copied from Platform's tests or produced by its DTO encoder.
const VIEW: &str = "4449444c086d016c06dbb70171bd869d8b0402998beb900602ddad9cd00903d9e5d5d60a06bbf8fded0f076e686e046c04d1a5c69c0471aea49cf50505e0e8dcb50b7ebbf8fded0f076e786e746d6801000108656e67696e652d780101010401010105010001ffffffffffffffff000201010201010301000000000000008002010101010102";
const WRONG_SUBNET_SOME: &str = "4449444c086d016c06dbb70171bd869d8b0402998beb900602ddad9cd00903d9e5d5d60a06bbf8fded0f076e686e716c04d1a5c69c0471aea49cf50505e0e8dcb50b7ebbf8fded0f076e786e746d6801000108656e67696e652d7800000101780000";
const SKIPPED_PREFIX: &str = "4449444c086d016c070071dbb70171bd869d8b0402998beb900602ddad9cd00903d9e5d5d60a06bbf8fded0f076e686e046c04d1a5c69c0471aea49cf50505e0e8dcb50b7ebbf8fded0f076e786e746d68010001";

fn hex(raw: &str) -> Vec<u8> {
    raw.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn id(n: u16) -> Principal {
    Principal::from_slice(&n.to_le_bytes())
}
fn request() -> EngineId {
    "engine-x".parse().unwrap()
}
fn view() -> wire::EngineView {
    wire::EngineView {
        id: "engine-x".into(),
        subnet_id: Some(id(20)),
        engine_operator_id: Some(id(21)),
        node_ids: vec![id(2), id(1)],
        deleted_at: None,
        subnet: Some(wire::EngineSubnet {
            node_ids: vec![id(1), id(3)],
            replica_version: "".into(),
            halted: false,
            maximum_state_size: Some(0),
        }),
    }
}
fn read(outcome: EngineOutcome) -> EngineRecord {
    let EngineOutcome::Read { record } = outcome else {
        panic!("expected admitted view")
    };
    *record
}
async fn literal(bytes: Vec<u8>) -> Result<EngineReport, EngineError> {
    collect(request(), |_| async {
        Ok(Observation {
            bytes,
            observed_at: 123,
        })
    })
    .await
}

#[test]
fn ids_are_bounded_opaque_utf8_before_io() {
    for raw in ["", &"x".repeat(257), &"é".repeat(129)] {
        let error = raw.parse::<EngineId>().unwrap_err();
        assert_eq!(error.bytes, raw.len());
        assert!(!error.to_string().contains("é"));
    }
    for raw in ["engine-x", " not-a-principal ", "\0\n", &"é".repeat(128)] {
        let parsed: EngineId = raw.parse().unwrap();
        assert_eq!(parsed.as_str(), raw);
        assert_eq!(serde_json::to_value(parsed).unwrap(), raw);
    }
}

#[tokio::test]
async fn independent_literal_retains_raw_values_and_exact_source() {
    let report = literal(hex(VIEW)).await.unwrap();
    assert_eq!(report.schema, 1);
    assert!(!report.atomic_snapshot);
    assert_eq!(report.evidence.source_canister, CANISTER);
    assert_eq!(report.evidence.source_method, METHOD);
    assert_eq!(report.evidence.endpoint, mainnet::ENDPOINT);
    assert_eq!(report.evidence.observed_at_unix_seconds, 123);
    assert!(report.evidence.query_signature_verified && !report.evidence.result_certified);
    let record = read(report.outcome);
    assert_eq!(record.deleted_at_unix_nanoseconds, Some(i64::MIN));
    let copy = record.registry_copy_subnet.unwrap();
    assert_eq!(copy.maximum_state_size_bytes, Some(u64::MAX));
    assert!(copy.configured_replica_version.is_empty());
    assert_eq!(
        record.record_node_ids,
        [Principal::from_slice(&[1]), Principal::from_slice(&[2])]
    );
    assert_eq!(
        copy.node_ids,
        [Principal::from_slice(&[2]), Principal::from_slice(&[3])]
    );
    let MembershipComparison::Compared {
        record_only,
        registry_copy_only,
    } = record.membership
    else {
        panic!()
    };
    assert_eq!(record_only, [Principal::from_slice(&[1])]);
    assert_eq!(registry_copy_only, [Principal::from_slice(&[3])]);
    assert!(matches!(
        literal(hex(WRONG_SUBNET_SOME)).await,
        Err(EngineError::Decode(decoder::Error::Type))
    ));
}

#[tokio::test]
async fn one_argument_one_query_and_successful_omission() {
    let called = Cell::new(0);
    let report = collect(request(), |argument| {
        called.set(called.get() + 1);
        // One text-vector argument; independently authored request frame.
        assert_eq!(argument, hex("4449444c016d7101000108656e67696e652d78"));
        async {
            Ok(Observation {
                bytes: encode_one(&Vec::<wire::EngineView>::new(), 1024).unwrap(),
                observed_at: 12,
            })
        }
    })
    .await
    .unwrap();
    assert_eq!(called.get(), 1);
    assert!(matches!(report.outcome, EngineOutcome::NotReturned));
    assert_eq!(
        serde_json::to_value(&report).unwrap()["outcome"]["status"],
        "not_returned"
    );
    let longest: EngineId = "é".repeat(128).parse().unwrap();
    collect(longest.clone(), |argument| async move {
        assert!(argument.len() <= 512);
        let decoded: Vec<String> = decoder::decode_one(&argument, LIMITS).unwrap();
        assert_eq!(decoded, [longest.as_str()]);
        Ok(Observation {
            bytes: encode_one(&Vec::<wire::EngineView>::new(), 1024).unwrap(),
            observed_at: 12,
        })
    })
    .await
    .unwrap();
}

#[test]
fn missing_and_empty_copy_differ_and_order_is_not_membership() {
    let mut input = view();
    input.subnet = None;
    let record = read(project(&request(), vec![input.clone()]).unwrap());
    assert!(record.registry_copy_subnet.is_none());
    assert!(matches!(
        record.membership,
        MembershipComparison::Unavailable
    ));
    input.subnet = Some(wire::EngineSubnet {
        node_ids: vec![],
        replica_version: "".into(),
        halted: true,
        maximum_state_size: None,
    });
    let record = read(project(&request(), vec![input.clone()]).unwrap());
    let MembershipComparison::Compared {
        record_only,
        registry_copy_only,
    } = record.membership
    else {
        panic!()
    };
    assert_eq!(record_only, [id(1), id(2)]);
    assert!(registry_copy_only.is_empty());
    input.subnet.as_mut().unwrap().node_ids = vec![id(1), id(2)];
    let record = read(project(&request(), vec![input]).unwrap());
    assert_eq!(record.record_node_ids, [id(2), id(1)]);
    let MembershipComparison::Compared {
        record_only,
        registry_copy_only,
    } = record.membership
    else {
        panic!()
    };
    assert!(record_only.is_empty() && registry_copy_only.is_empty());
    for deleted in [None, Some(0), Some(-1), Some(i64::MAX)] {
        let mut input = view();
        input.deleted_at = deleted;
        let record = read(project(&request(), vec![input]).unwrap());
        assert_eq!(record.deleted_at_unix_nanoseconds, deleted);
        assert_eq!(
            record
                .registry_copy_subnet
                .unwrap()
                .maximum_state_size_bytes,
            Some(0)
        );
    }
}

#[test]
fn identity_count_field_and_relationship_refusals_are_explicit() {
    assert!(matches!(
        project(&request(), vec![view(), view()]),
        Err(EngineRefusal::Limit { field: "views", .. })
    ));
    let mut wrong = view();
    wrong.id = "Engine-x".into();
    assert!(matches!(
        project(&request(), vec![wrong]),
        Err(EngineRefusal::WrongEngine)
    ));
    let mut too_long = view();
    too_long.id = "x".repeat(257);
    assert!(matches!(
        project(&request(), vec![too_long]),
        Err(EngineRefusal::Limit {
            field: "id_bytes",
            ..
        })
    ));
    let mut inconsistent = view();
    inconsistent.subnet_id = None;
    assert!(matches!(
        project(&request(), vec![inconsistent]),
        Err(EngineRefusal::MissingSubnetId)
    ));
    for field in ["subnet", "operator", "record", "copy"] {
        let mut input = view();
        let empty = Principal::from_slice(&[]);
        match field {
            "subnet" => input.subnet_id = Some(empty),
            "operator" => input.engine_operator_id = Some(empty),
            "record" => input.node_ids.push(empty),
            _ => input.subnet.as_mut().unwrap().node_ids.push(empty),
        }
        assert!(matches!(
            project(&request(), vec![input]),
            Err(EngineRefusal::EmptyIdentity { .. })
        ));
    }
    for copy in [false, true] {
        let mut input = view();
        let list = if copy {
            &mut input.subnet.as_mut().unwrap().node_ids
        } else {
            &mut input.node_ids
        };
        list.push(list[0]);
        assert!(matches!(
            project(&request(), vec![input]),
            Err(EngineRefusal::DuplicateNode { .. })
        ));
        for count in [128, 129] {
            let mut input = view();
            let list = if copy {
                &mut input.subnet.as_mut().unwrap().node_ids
            } else {
                &mut input.node_ids
            };
            *list = (0..count).map(id).collect();
            let result = project(&request(), vec![input]);
            assert_eq!(result.is_ok(), count == 128);
        }
    }
    for bytes in [256, 258] {
        let mut input = view();
        input.subnet.as_mut().unwrap().replica_version = "é".repeat(bytes / 2);
        assert_eq!(project(&request(), vec![input]).is_ok(), bytes == 256);
    }
    let mut empty = view();
    empty.subnet = None;
    empty.subnet_id = None;
    empty.engine_operator_id = None;
    empty.node_ids.clear();
    assert!(project(&request(), vec![empty]).is_ok());
}

#[tokio::test]
async fn published_decode_budgets_and_skipped_fields_remain_bounded() {
    let frame = hex(VIEW);
    for prefix in [0, 4, frame.len() - 1] {
        assert!(matches!(
            literal(frame[..prefix].to_vec()).await,
            Err(EngineError::Decode(_))
        ));
    }
    let mut trailing = frame;
    trailing.push(0);
    assert!(matches!(
        literal(trailing).await,
        Err(EngineError::Decode(decoder::Error::Trailing))
    ));
    assert!(matches!(
        literal(vec![0; LIMITS.bytes + 1]).await,
        Err(EngineError::Decode(decoder::Error::Limit))
    ));
    for length in [100_000usize, 140_000] {
        // Extra field label 0:text, followed by the selected fields. Construct only
        // its literal length/payload, with no second external DTO or local codec.
        let mut frame = hex(SKIPPED_PREFIX);
        let mut n = length;
        loop {
            let byte = (n & 127) as u8;
            n >>= 7;
            frame.push(byte | if n > 0 { 128 } else { 0 });
            if n == 0 {
                break;
            }
        }
        frame.extend(std::iter::repeat_n(b'x', length));
        frame.extend(hex("08656e67696e652d780000000000"));
        let result = literal(frame).await;
        if length == 100_000 {
            let report = result.unwrap();
            assert!(
                !serde_json::to_string(&report)
                    .unwrap()
                    .contains(&"x".repeat(20))
            );
        } else {
            assert!(matches!(
                result,
                Err(EngineError::Decode(decoder::Error::Limit))
            ));
        }
    }
    let mut input = view();
    input.node_ids = (0..128).map(id).collect();
    input.subnet.as_mut().unwrap().node_ids = (128..256).map(id).collect();
    input.subnet.as_mut().unwrap().replica_version = "v".repeat(256);
    let report = literal(encode_one(&vec![input], LIMITS.bytes).unwrap())
        .await
        .unwrap();
    assert_eq!(read(report.outcome).record_node_ids.len(), 128);
}

#[tokio::test(start_paused = true)]
async fn shared_deadline_errors_and_cancellation_do_not_retry() {
    let calls = Cell::new(0);
    let result = collect(request(), |_| async {
        calls.set(calls.get() + 1);
        mainnet::read::<Vec<u8>>(std::future::pending())
            .await
            .map(|bytes| Observation {
                bytes,
                observed_at: 0,
            })
            .map_err(EngineError::Read)
    })
    .await;
    assert!(matches!(
        result,
        Err(EngineError::Read(ReadError::Deadline))
    ));
    assert_eq!(calls.get(), 1);
    struct Dropped<'a>(&'a Cell<bool>);
    impl Drop for Dropped<'_> {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    let dropped = Cell::new(false);
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        collect(request(), |_| async {
            let _guard = Dropped(&dropped);
            std::future::pending().await
        }),
    )
    .await;
    assert!(result.is_err() && dropped.get());
    let result = collect(request(), |_| async {
        Err(EngineError::Read(ReadError::Transport(Box::new(
            ic_platform::agent::AgentError::MessageError("fixture".into()),
        ))))
    })
    .await;
    assert!(matches!(
        result,
        Err(EngineError::Read(ReadError::Transport(_)))
    ));
}
