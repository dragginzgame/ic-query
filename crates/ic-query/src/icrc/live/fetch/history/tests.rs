use super::*;
use crate::icrc::live::tests::{query_content, replied_query_fixture};
use ic_platform::icrc_ledger_types::QueryArchiveFn;
use std::collections::BTreeMap;

#[test]
fn wire_history_projects_plain_decimal_numbers_accepted_by_report_validation() {
    let large: Nat = "18446744073709551616".parse().unwrap();
    let block = BlockWithId {
        id: Nat::from(1_000u32),
        block: ICRC3Value::Map(BTreeMap::from([
            ("ts".into(), ICRC3Value::Nat(large.clone())),
            (
                "tx".into(),
                ICRC3Value::Map(BTreeMap::from([(
                    "amt".into(),
                    ICRC3Value::Nat(large.clone()),
                )])),
            ),
            (
                "signed".into(),
                ICRC3Value::Int(ic_platform::candid::contract::Int::from(-1_000)),
            ),
            ("blob".into(), ICRC3Value::Blob(vec![0, 0xab, 0xff].into())),
            (
                "array".into(),
                ICRC3Value::Array(vec![ICRC3Value::Text("fixture".into())]),
            ),
        ])),
    };
    let result = GetBlocksResult {
        log_length: large,
        blocks: vec![block.clone()],
        archived_blocks: vec![ArchivedBlocks {
            args: vec![GetBlocksRequest {
                start: Nat::from(1_001u32),
                length: Nat::from(1_000u32),
            }],
            callback: QueryArchiveFn::new(Principal::anonymous(), "read_page"),
        }],
    };
    let request = IcrcTransactionsRequest {
        ledger: crate::icrc::IcrcLedgerRequest::new("fixture", 0, Principal::anonymous().to_text()),
        start: 1_000,
        limit: 1_001,
        follow_archives: true,
    };
    validate_block_page(&result, &request).unwrap();
    let mut followed = block;
    followed.id = Nat::from(1_001u32);
    let data = transactions_data_from_blocks(
        result,
        ArchiveFollowResult {
            blocks: vec![followed_archive_block_row_from_wire(
                &Principal::anonymous().to_text(),
                "read_page",
                followed,
            )],
            errors: vec![],
        },
    );
    validate_transactions_data(&request, &data).expect("wire data satisfies report contract");
    assert_eq!(data.blocks[0].raw_block["Map"]["blob"]["Blob"], "00abff");
    assert_eq!(
        data.blocks[0].raw_block["Map"]["array"]["Array"][0]["Text"],
        "fixture"
    );
    assert_eq!(data.log_length.as_deref(), Some("18446744073709551616"));
    assert_eq!(data.blocks[0].index, "1000");
    assert_eq!(data.followed_archive_blocks[0].block.index, "1001");
    assert_eq!(data.archived_blocks[0].ranges[0].start, "1001");
    assert_eq!(data.archived_blocks[0].ranges[0].length, "1000");
    assert_eq!(
        data.blocks[0].amount_base_units.as_deref(),
        Some("18446744073709551616")
    );
    assert_eq!(
        data.blocks[0].timestamp_unix_nanos.as_deref(),
        Some("18446744073709551616")
    );
    assert_eq!(
        data.blocks[0].raw_block["Map"]["tx"]["Map"]["amt"]["Nat"],
        "18446744073709551616"
    );
    assert_eq!(data.blocks[0].raw_block["Map"]["signed"]["Int"], "-1000");
    let archive = archive_row_from_wire(ICRC3ArchiveInfo {
        canister_id: Principal::anonymous(),
        start: Nat::from(1_000u32),
        end: Nat::from(2_000u32),
    });
    assert_eq!(archive.start, "1000");
    assert_eq!(archive.end, "2000");
}

#[test]
fn followed_blocks_remain_bound_to_each_callbacks_disjoint_ranges() {
    let canister = Principal::anonymous();
    let archive = |start: u32, method: &str| ArchivedBlocks {
        args: vec![GetBlocksRequest {
            start: Nat::from(start),
            length: Nat::from(1u32),
        }],
        callback: QueryArchiveFn::new(canister, method),
    };
    let followed = |index: u32, method: &str| {
        followed_archive_block_row_from_wire(
            &canister.to_text(),
            method,
            BlockWithId {
                id: Nat::from(index),
                block: ICRC3Value::Nat(Nat::from(index)),
            },
        )
    };
    let request = IcrcTransactionsRequest {
        ledger: crate::icrc::IcrcLedgerRequest::new("fixture", 0, canister.to_text()),
        start: 0,
        limit: 4,
        follow_archives: true,
    };
    let data = transactions_data_from_blocks(
        GetBlocksResult {
            log_length: Nat::from(4u32),
            blocks: vec![],
            archived_blocks: vec![
                archive(0, "first"),
                archive(2, "first"),
                archive(1, "second"),
                archive(3, "second"),
            ],
        },
        ArchiveFollowResult {
            blocks: vec![
                followed(0, "first"),
                followed(2, "first"),
                followed(1, "second"),
                followed(3, "second"),
            ],
            errors: vec![],
        },
    );
    validate_transactions_data(&request, &data)
        .expect("all ranges for the same callback are retained");
    let mut wrong_canister = followed(0, "first");
    wrong_canister.archive_canister_id = Principal::management_canister().to_text();
    for rejected in [followed(0, "second"), followed(4, "second"), wrong_canister] {
        let mut invalid = data.clone();
        invalid.followed_archive_blocks = vec![rejected];
        assert!(matches!(
            validate_transactions_data(&request, &invalid),
            Err(IcrcError::InvalidTransactionPage { .. })
        ));
    }
    let mut duplicated = data;
    duplicated
        .followed_archive_blocks
        .push(followed(0, "first"));
    assert!(matches!(
        validate_transactions_data(&request, &duplicated),
        Err(IcrcError::InvalidTransactionPage { .. })
    ));
}

async fn follow_callback_fixture(block_ids: &[u64]) -> ArchiveFollowResult {
    let reply = GetBlocksResult {
        log_length: Nat::from(100u64),
        blocks: block_ids
            .iter()
            .map(|id| BlockWithId {
                id: Nat::from(*id),
                block: ICRC3Value::Text("fixture".into()),
            })
            .collect(),
        archived_blocks: vec![],
    };
    let (server, agent) = replied_query_fixture(
        &ic_platform::candid::contract::encode_one(reply).unwrap(),
        8 * 1024 * 1024,
    );
    let archive = ArchivedBlocks {
        callback: QueryArchiveFn::new(Principal::anonymous(), "read_archive_page"),
        args: vec![GetBlocksRequest {
            start: Nat::from(12u64),
            length: Nat::from(2u64),
        }],
    };
    let followed = fetch_archive_blocks(&agent, &[archive]).await;
    let raw = server.finish().unwrap();
    let ic_platform::ic_protocol::EnvelopeContent::Query {
        method_name, arg, ..
    } = query_content(&raw)
    else {
        panic!("query content")
    };
    assert_eq!(method_name, "read_archive_page");
    let args: Vec<GetBlocksRequest> = crate::candid_decode::decode_reply(&arg).unwrap();
    assert_eq!(args[0].start, Nat::from(12u64));
    assert_eq!(args[0].length, Nat::from(2u64));
    followed
}

#[tokio::test(flavor = "current_thread")]
async fn follows_the_supplied_query_callback_method_and_arguments() {
    let followed = follow_callback_fixture(&[12]).await;
    assert!(followed.errors.is_empty(), "{:?}", followed.errors);
    assert_eq!(followed.blocks[0].block.index, "12");
    assert_eq!(followed.blocks[0].callback_method, "read_archive_page");
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_archive_rows_are_reported_without_retaining_partial_blocks() {
    for ids in [&[12, 14][..], &[12, 12][..]] {
        let followed = follow_callback_fixture(ids).await;
        assert_eq!(followed.blocks, Vec::<IcrcFollowedArchiveBlockRow>::new());
        assert_eq!(followed.errors.len(), 1);
        assert_eq!(followed.errors[0].callback_method, "read_archive_page");
        assert_eq!(followed.errors[0].ranges[0].start, "12");
    }
}

#[test]
fn archive_reply_budget_counts_every_reply_and_stops_at_exhaustion() {
    let mut remaining = 10;
    charge_archive_reply(&mut remaining, 4).unwrap();
    assert_eq!(remaining, 6);
    assert!(matches!(
        charge_archive_reply(&mut remaining, 7),
        Err(IcrcError::InvalidTransactionPage { .. })
    ));
    assert_eq!(remaining, 0);
    assert!(charge_archive_reply(&mut remaining, 1).is_err());
    let mut exact = 4;
    charge_archive_reply(&mut exact, 4).unwrap();
    assert_eq!(exact, 0);
}

#[tokio::test(flavor = "current_thread")]
async fn exhausted_archive_budget_rejects_work_before_querying() {
    let agent = Agent::builder()
        .with_url("http://127.0.0.1:1")
        .build()
        .unwrap();
    let archive = ArchivedBlocks {
        args: vec![GetBlocksRequest {
            start: Nat::from(12u32),
            length: Nat::from(1u32),
        }],
        callback: QueryArchiveFn::new(Principal::anonymous(), "read_page"),
    };
    let mut remaining = 0;
    let error = query_archive_blocks(&agent, &archive, &mut remaining)
        .await
        .expect_err("no network work after budget exhaustion");
    assert!(matches!(error, IcrcError::InvalidTransactionPage { .. }));
}

#[test]
fn block_page_validates_request_and_log_bounds_and_disjoint_coverage() {
    let request = IcrcTransactionsRequest {
        ledger: crate::icrc::IcrcLedgerRequest::new("fixture", 0, Principal::anonymous().to_text()),
        start: 12,
        limit: 2,
        follow_archives: true,
    };
    let range = |start, length| GetBlocksRequest {
        start: Nat::from(start),
        length: Nat::from(length),
    };
    let archive = |args| ArchivedBlocks {
        args,
        callback: QueryArchiveFn::new(Principal::anonymous(), "read_page"),
    };
    let mut result = GetBlocksResult {
        log_length: Nat::from(100u32),
        blocks: vec![],
        archived_blocks: vec![archive(vec![range(12u64, 2u64)])],
    };
    validate_block_page(&result, &request).unwrap();
    for length in [0u32, 12, 13] {
        result.log_length = Nat::from(length);
        assert!(matches!(
            validate_block_page(&result, &request),
            Err(IcrcError::InvalidTransactionPage { .. })
        ));
    }
    result.log_length = Nat::from(14u32);
    validate_block_page(&result, &request).unwrap();
    for args in [
        vec![],
        vec![range(12, 0)],
        vec![range(11, 2)],
        vec![range(13, 2)],
        vec![range(12, 1), range(12, 1)],
    ] {
        result.archived_blocks = vec![archive(args)];
        assert!(matches!(
            validate_block_page(&result, &request),
            Err(IcrcError::InvalidTransactionPage { .. })
        ));
    }
    result.archived_blocks = vec![archive(vec![range(12, 2)])];
    result.blocks.push(BlockWithId {
        id: Nat::from(12u32),
        block: ICRC3Value::Text("overlap".into()),
    });
    assert!(validate_block_page(&result, &request).is_err());
    result.blocks.clear();
    result.archived_blocks = (0..=MAX_ARCHIVE_CALLBACKS)
        .map(|_| archive(vec![range(12, 1)]))
        .collect();
    assert!(validate_block_page(&result, &request).is_err());
    result.archived_blocks.clear();
    result.blocks = vec![BlockWithId {
        id: Nat::from(14u32),
        block: ICRC3Value::Text("outside".into()),
    }];
    assert!(validate_block_page(&result, &request).is_err());
    result.blocks[0].id = Nat::from(13u32);
    result.log_length = Nat::from(13u32);
    assert!(matches!(
        validate_block_page(&result, &request),
        Err(IcrcError::InvalidTransactionPage { .. })
    ));
    result.log_length = Nat::from(14u32);
    validate_block_page(&result, &request).unwrap();
    let huge_request = IcrcTransactionsRequest {
        start: u64::MAX,
        ..request
    };
    result.blocks[0].id = Nat(Nat::from(u64::MAX).0 + 1u32);
    result.log_length = Nat(Nat::from(u64::MAX).0 + 2u32);
    validate_block_page(&result, &huge_request).unwrap();
    result.blocks.clear();
    result.log_length = Nat::from(0u32);
    validate_block_page(&result, &huge_request).expect("empty page past the end of the log");
}
