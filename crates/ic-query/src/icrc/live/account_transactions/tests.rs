use super::*;
use crate::{QueryProgressEvent, icrc::IcrcAccountTransactionCacheRequest};
use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
use ic_platform::candid::contract::{CandidType, Nat};
use serde_cbor::Value;
use std::{collections::BTreeMap, time::Duration};

#[derive(CandidType)]
#[candid_path("ic_platform::candid::contract")]
struct IndexPage {
    balance: Nat,
    transactions: Vec<IndexTransaction>,
    oldest_tx_id: Option<Nat>,
}

#[derive(CandidType)]
#[candid_path("ic_platform::candid::contract")]
struct IndexTransaction {
    id: Nat,
    transaction: Transaction,
}

#[derive(CandidType)]
#[candid_path("ic_platform::candid::contract")]
struct Transaction {
    kind: String,
    timestamp: u64,
}

#[derive(CandidType)]
#[candid_path("ic_platform::candid::contract")]
struct IndexError {
    message: String,
}

fn page_reply(ids: &[u64], oldest: Option<u64>) -> Vec<u8> {
    let response: Result<IndexPage, IndexError> = Ok(IndexPage {
        balance: Nat::from(100_u64),
        transactions: ids
            .iter()
            .map(|id| IndexTransaction {
                id: Nat::from(*id),
                transaction: Transaction {
                    kind: "fixture".to_string(),
                    timestamp: 1,
                },
            })
            .collect(),
        oldest_tx_id: oldest.map(Nat::from),
    });
    ic_platform::candid::contract::encode_one(response).unwrap()
}

async fn run_page(
    reply: Vec<u8>,
    max_pages: Option<u32>,
    accepted_page: Option<AccountTransactionsPage>,
) -> (
    Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError>,
    Vec<QueryProgressEvent>,
) {
    let envelope = Value::Map(BTreeMap::from([
        (Value::Text("status".into()), Value::Text("replied".into())),
        (
            Value::Text("reply".into()),
            Value::Map(BTreeMap::from([(
                Value::Text("arg".into()),
                Value::Bytes(reply),
            )])),
        ),
    ]));
    let server = HttpFixture::spawn(
        &serde_cbor::to_vec(&envelope).unwrap(),
        "application/cbor",
        HttpFixtureLimits {
            timeout: Duration::from_secs(10),
            max_header_bytes: 64 * 1024,
            max_body_bytes: 64 * 1024,
        },
    )
    .unwrap();
    let context = AccountTransactionQueryContext {
        agent: Agent::builder()
            .with_url(server.endpoint())
            .with_verify_query_signatures(false)
            .build()
            .unwrap(),
        index_canister: Principal::management_canister(),
        account: LedgerAccount {
            owner: Principal::anonymous(),
            subaccount: None,
        },
        token_symbol: "TEST".to_string(),
        decimals: 8,
    };
    let request = IcrcAccountTransactionRefreshRequest {
        cache: IcrcAccountTransactionCacheRequest::new(
            "/unused",
            server.endpoint(),
            "aaaaa-aa",
            Principal::anonymous().to_text(),
        ),
        index_canister_id: None,
        page_size: 2,
        max_pages,
        now_unix_secs: 1_700_000_000,
    };
    let mut state = AccountTransactionCollectionState::new(context.index_canister.to_text());
    if let Some(page) = accepted_page {
        assert!(!state.ingest(page, request.page_size).unwrap());
    }
    let mut events = Vec::new();
    let result = run_paged_snapshot_refresh_with_progress(
        AccountTransactionRefresh {
            request: &request,
            context: &context,
            state,
        },
        &mut |event| events.push(event),
    )
    .await
    .and_then(|state| state.into_complete(context.token_symbol, context.decimals));
    server.finish().unwrap();
    (result, events)
}

#[tokio::test(flavor = "current_thread")]
async fn paging_preserves_empty_and_nonempty_exhaustion_and_progress_text() {
    for (ids, oldest) in [(vec![], None), (vec![9], Some(9))] {
        let (result, events) = run_page(page_reply(&ids, oldest), None, None).await;
        let data = result.unwrap();
        assert_eq!(data.transactions.len(), ids.len());
        assert_eq!(data.page_count, 1);
        assert_eq!(data.balance, "100");
        assert_eq!(
            events,
            [
                QueryProgressEvent::PagedRefresh {
                    text: "refreshing ICRC account transactions: pages=0 rows=0".to_string(),
                    state: QueryProgressState::Running,
                },
                QueryProgressEvent::PagedRefresh {
                    text: format!(
                        "refreshing ICRC account transactions: pages=1 rows={}",
                        ids.len()
                    ),
                    state: QueryProgressState::Running,
                },
                QueryProgressEvent::PagedRefresh {
                    text: format!(
                        "refreshing ICRC account transactions: pages=1 rows={}",
                        ids.len()
                    ),
                    state: QueryProgressState::Complete,
                },
            ]
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn page_ceiling_retains_index_cursor_and_diagnostic_before_another_fetch() {
    let (result, events) = run_page(page_reply(&[9], Some(1)), Some(1), None).await;
    assert!(
        matches!(result, Err(IcrcAccountTransactionError::IncompleteCollection {
        index_canister_id: Some(index), pages_fetched: 1, rows_fetched: 1,
        last_cursor: Some(cursor), reason,
    }) if index == "aaaaa-aa" && cursor == "9" && reason == "max pages reached before index exhaustion")
    );
    assert!(
        matches!(events.last(), Some(QueryProgressEvent::PagedRefresh {
        state: QueryProgressState::Stopped, text,
    }) if text == "refreshing ICRC account transactions: pages=1 rows=1")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_pages_fail_before_advancing_collection_progress() {
    let (result, events) = run_page(page_reply(&[9, 10], Some(1)), None, None).await;
    assert!(matches!(
        result,
        Err(IcrcAccountTransactionError::IncompleteCollection {
            pages_fetched: 0,
            rows_fetched: 0,
            last_cursor: None,
            ..
        })
    ));
    assert!(
        matches!(events.last(), Some(QueryProgressEvent::PagedRefresh {
        state: QueryProgressState::Failed, text,
    }) if text == "refreshing ICRC account transactions: pages=0 rows=0")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn query_failures_retain_previously_accepted_progress_and_original_error() {
    let accepted = AccountTransactionsPage {
        balance: "100".to_string(),
        oldest_transaction_id: Some("1".to_string()),
        transactions: vec![
            serde_json::from_value(serde_json::json!({
                "id": "9", "kind": "fixture", "raw_transaction": {},
            }))
            .unwrap(),
        ],
    };
    let (result, events) = run_page(vec![0xff], None, Some(accepted)).await;
    assert!(
        matches!(result, Err(IcrcAccountTransactionError::CollectionPage {
        index_canister_id: Some(index), pages_fetched: 1, rows_fetched: 1,
        last_cursor: Some(cursor), source,
    }) if index == "aaaaa-aa" && cursor == "9" && matches!(*source,
        IcrcAccountTransactionError::Query(IcrcError::CandidDecode { .. })))
    );
    assert!(
        matches!(events.last(), Some(QueryProgressEvent::PagedRefresh {
        state: QueryProgressState::Failed, text,
    }) if text == "refreshing ICRC account transactions: pages=1 rows=1")
    );
}
