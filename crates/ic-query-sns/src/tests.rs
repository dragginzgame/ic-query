//! Consumer regression cases using frozen independent supplier wire fixtures.
use crate::{
    balance,
    mainnet::{self, Source},
    neuron::{self, ListRequest, StopReason},
    *,
};
use ic_platform_protocol::sns;
use ic_platform_types::{Account, Principal};
use std::{cell::RefCell, collections::VecDeque, time::Duration};

struct Fake {
    replies: RefCell<VecDeque<Result<Vec<u8>, Failure>>>,
    calls: RefCell<Vec<(&'static str, Vec<u8>)>>,
    delay: Duration,
}

impl Fake {
    fn new(replies: Vec<Result<Vec<u8>, Failure>>) -> Self {
        Self {
            replies: RefCell::new(replies.into()),
            calls: RefCell::new(vec![]),
            delay: Duration::ZERO,
        }
    }
}

impl Source for Fake {
    async fn query(
        &self,
        _: Principal,
        method: &'static str,
        args: &[u8],
    ) -> Result<Vec<u8>, Failure> {
        self.calls.borrow_mut().push((method, args.to_vec()));
        tokio::time::sleep(self.delay).await;
        self.replies
            .borrow_mut()
            .pop_front()
            .expect("unexpected extra read")
    }
}

fn failed() -> Failure {
    Failure::new(FailureKind::Read, "fixture unavailable")
}

fn target() -> Principal {
    "zqfso-syaaa-aaaaq-aaafq-cai".parse().unwrap()
}

fn fixture_neuron() -> neuron::Neuron {
    let Some(sns::GetNeuronResult::Neuron(n)) =
        sns::decode_get_neuron(include_bytes!("fixtures/sns-neuron.bin"), mainnet::LIMITS)
            .unwrap()
            .result
    else {
        panic!()
    };
    n
}

fn request(pages: u8) -> ListRequest {
    ListRequest {
        governance: target(),
        of_principal: None,
        start_after: None,
        max_pages: pages,
    }
}

fn page(start: u8, length: usize) -> Vec<neuron::Neuron> {
    (0..length)
        .map(|i| {
            let mut n = fixture_neuron();
            n.id = Some(sns::NeuronId {
                id: vec![start + u8::try_from(i).unwrap(); 32],
            });
            n
        })
        .collect()
}

fn observed<T>(value: T) -> Observation<T> {
    Observation {
        method: "list_neurons",
        started_at_unix_ms: Some(1),
        finished_at_unix_ms: Some(2),
        query_signature_verified: true,
        result_certified: false,
        outcome: Outcome::Read { value },
    }
}

#[tokio::test]
async fn balance_keeps_above_u128_and_independent_metadata_failure() {
    let fake = Fake::new(vec![
        Ok(include_bytes!("fixtures/icrc-balance.bin").to_vec()),
        Err(failed()),
        Ok(include_bytes!("fixtures/icrc-symbol.bin").to_vec()),
    ]);
    let account = Account {
        owner: Principal::anonymous(),
        subaccount: Some([0; 32]),
    };
    let report = balance::collect(&fake, target(), account).await;
    assert!(!report.complete);
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(
        json["balance"]["outcome"]["value"],
        "340282366920938463463374607431768211456"
    );
    assert_eq!(json["symbol"]["outcome"]["value"], "TÖKO\0");
    assert_eq!(json["decimals"]["outcome"]["status"], "failed");
    assert_eq!(report.account.subaccount, Some([0; 32]));
    assert_eq!(
        fake.calls.borrow().iter().map(|c| c.0).collect::<Vec<_>>(),
        ["icrc1_balance_of", "icrc1_decimals", "icrc1_symbol"]
    );
    assert!(!report.atomic_snapshot);
    assert!(report.balance.query_signature_verified);
    assert!(!report.balance.result_certified);
}

#[tokio::test]
async fn failed_balance_is_not_zero_and_does_not_hide_metadata() {
    let fake = Fake::new(vec![
        Err(failed()),
        Ok(b"DIDL\0\x01\x7b\x08".to_vec()),
        Ok(include_bytes!("fixtures/icrc-symbol.bin").to_vec()),
    ]);
    let report = balance::collect(&fake, target(), Principal::anonymous().into()).await;
    assert!(!report.balance.is_read());
    assert!(!report.balance.query_signature_verified);
    assert!(report.decimals.is_read() && report.symbol.is_read());
    let json = serde_json::to_value(report).unwrap();
    assert!(json["balance"]["outcome"].get("value").is_none());
    assert!(json["account"]["subaccount"].is_null());
}

#[tokio::test]
async fn optional_presence_and_unknown_permission_codes_survive_get() {
    let fake = Fake::new(vec![Ok(include_bytes!("fixtures/sns-neuron.bin").to_vec())]);
    let report = neuron::get(&fake, target(), [255; 32]).await;
    let Outcome::Read { value } = report.neuron.outcome else {
        panic!()
    };
    assert_eq!(value.cached_neuron_stake_e8s, u64::MAX);
    assert_eq!(value.permissions[0].principal, None);
    assert_eq!(
        value.permissions[0].permission_type,
        [i32::MIN, -1, 0, i32::MAX]
    );
    assert_eq!(value.staked_maturity_e8s_equivalent, Some(0));
    assert_eq!(value.auto_stake_maturity, Some(false));
    assert_eq!(value.neuron_fees_e8s, 17);
    assert_eq!(value.maturity_e8s_equivalent, 23);
}

#[tokio::test]
async fn get_separates_wrong_id_missing_result_governance_and_decode_failures() {
    for (bytes, expected) in [
        (
            include_bytes!("fixtures/sns-neuron.bin").as_slice(),
            FailureKind::Admission,
        ),
        (
            include_bytes!("fixtures/sns-missing.bin").as_slice(),
            FailureKind::MissingResult,
        ),
        (
            include_bytes!("fixtures/sns-error.bin").as_slice(),
            FailureKind::Governance,
        ),
        (b"not candid".as_slice(), FailureKind::Decode),
    ] {
        let fake = Fake::new(vec![Ok(bytes.to_vec())]);
        let report = neuron::get(&fake, target(), [1; 32]).await;
        let Outcome::Failed { error } = report.neuron.outcome else {
            panic!()
        };
        assert_eq!(error.kind, expected);
        assert_eq!(
            error.governance_error_type.is_some(),
            expected == FailureKind::Governance
        );
    }
}

#[tokio::test]
async fn malformed_neuron_page_does_not_become_empty_success() {
    // Independent list fixture includes one valid neuron and one absent ID.
    let fake = Fake::new(vec![Ok(include_bytes!("fixtures/sns-list.bin").to_vec())]);
    let report = neuron::collect(&fake, request(2)).await.unwrap();
    assert_eq!(report.stop_reason, StopReason::Failed);
    assert_eq!(report.retained_neurons, 0);
    assert!(!report.traversal_exhausted);
    assert_eq!(fake.calls.borrow().len(), 1);
}

#[tokio::test]
async fn page_limit_and_short_page_have_distinct_coverage() {
    let report = neuron::collect_pages(request(1), |cursor, _| {
        assert!(cursor.is_none());
        std::future::ready(observed(page(1, 100)))
    })
    .await
    .unwrap();
    assert_eq!(report.stop_reason, StopReason::PageLimit);
    assert!(!report.traversal_exhausted);
    let mut calls = 0;
    let report = neuron::collect_pages(request(3), |cursor, _| {
        calls += 1;
        let rows = if calls == 1 {
            assert!(cursor.is_none());
            page(1, 100)
        } else {
            assert_eq!(cursor, Some([100; 32]));
            page(101, 2)
        };
        std::future::ready(observed(rows))
    })
    .await
    .unwrap();
    assert_eq!(calls, 2);
    assert_eq!(report.retained_neurons, 102);
    assert_eq!(report.pages[1].start_after, Some([100; 32]));
    assert_eq!(report.stop_reason, StopReason::Exhausted);
    assert!(!report.atomic_snapshot);
}

#[tokio::test]
async fn later_failure_or_repeated_cursor_preserves_prior_pages() {
    for repeats in [false, true] {
        let mut calls = 0;
        let report = neuron::collect_pages(request(3), |_, _| {
            calls += 1;
            let mut o = observed(page(1, 100));
            if calls == 2 && !repeats {
                o.outcome = Outcome::Failed { error: failed() };
            }
            std::future::ready(o)
        })
        .await
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(report.retained_neurons, 100);
        assert_eq!(report.stop_reason, StopReason::Failed);
        assert!(report.pages[0].observation.is_read());
        assert!(!report.pages[1].observation.is_read());
    }
}

#[tokio::test]
async fn filtered_full_page_is_not_exhausted_or_paginated() {
    let mut request = request(1);
    request.of_principal = Some(Principal::anonymous());
    let report = neuron::collect_pages(request, |_, _| std::future::ready(observed(page(1, 100))))
        .await
        .unwrap();
    assert_eq!(report.stop_reason, StopReason::FilterLimit);
    assert_eq!(report.pages.len(), 1);
    assert!(!report.traversal_exhausted);
    request.max_pages = 2;
    assert!(
        neuron::collect_pages(request, |_, _| async { panic!("must not read") })
            .await
            .is_err()
    );
}

#[test]
fn pages_refuse_duplicates_nonadvance_invalid_ids_and_wrong_filter() {
    assert!(neuron::validate_page(&page(1, 101), None, None).is_err());
    assert!(neuron::validate_page(&[fixture_neuron(), fixture_neuron()], None, None).is_err());
    assert!(neuron::validate_page(&page(1, 2), Some([1; 32]), None).is_err());
    let mut unordered = page(1, 2);
    unordered.reverse();
    assert!(neuron::validate_page(&unordered, None, None).is_err());
    assert!(neuron::validate_page(&unordered, None, Some(Principal::anonymous())).is_ok());
    assert!(neuron::validate_page(&unordered, None, Some(target())).is_err());
    for id in [None, Some(sns::NeuronId { id: vec![1; 31] })] {
        let mut n = fixture_neuron();
        n.id = id;
        assert!(neuron::validate_page(&[n], None, None).is_err());
    }
}

#[tokio::test(start_paused = true)]
async fn leaf_timeout_and_shared_deadline_prevent_extra_calls() {
    let mut fake = Fake::new(vec![]);
    fake.delay = Duration::from_secs(25);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(21);
    let first = mainnet::observe(&fake, target(), "get_neuron", Ok(vec![]), deadline).await;
    let second = mainnet::observe(&fake, target(), "get_neuron", Ok(vec![]), deadline).await;
    let third = mainnet::observe(&fake, target(), "get_neuron", Ok(vec![]), deadline).await;
    for o in [first, second, third] {
        assert!(matches!(
            o.outcome,
            Outcome::Failed {
                error: Failure {
                    kind: FailureKind::Deadline,
                    ..
                }
            }
        ));
    }
    assert_eq!(fake.calls.borrow().len(), 2);
}

#[tokio::test(start_paused = true)]
async fn cancellation_starts_no_subsequent_metadata_reads() {
    let mut fake = Fake::new(vec![]);
    fake.delay = Duration::from_secs(25);
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        balance::collect(&fake, target(), Principal::anonymous().into()),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(fake.calls.borrow().len(), 1);
}

#[tokio::test]
async fn explicit_continuation_binds_first_cursor_and_rejects_filter_combination() {
    let mut r = request(1);
    r.start_after = Some([100; 32]);
    let report = neuron::collect_pages(r, |cursor, _| {
        assert_eq!(cursor, Some([100; 32]));
        std::future::ready(observed(page(101, 1)))
    })
    .await
    .unwrap();
    assert_eq!(report.pages[0].start_after, Some([100; 32]));
    assert!(report.traversal_exhausted);
    r.of_principal = Some(Principal::anonymous());
    assert!(
        neuron::collect_pages(r, |_, _| async { panic!("must not read") })
            .await
            .is_err()
    );
}

#[tokio::test]
async fn decimal_expansion_and_symbol_size_refuse_without_truncation() {
    // Literal one-Nat Candid frame containing a 4103-bit power of two.
    let mut amount = b"DIDL\0\x01\x7d".to_vec();
    amount.extend(std::iter::repeat_n(0x80, 586));
    amount.push(1);
    // Literal one-text Candid frame containing 257 ASCII bytes.
    let mut symbol = b"DIDL\0\x01\x71\x81\x02".to_vec();
    symbol.extend(std::iter::repeat_n(b'x', 257));
    let fake = Fake::new(vec![
        Ok(amount),
        Ok(b"DIDL\0\x01\x7b\xff".to_vec()),
        Ok(symbol),
    ]);
    let report = balance::collect(&fake, target(), Principal::anonymous().into()).await;
    for failed in [&report.balance, &report.symbol] {
        assert!(matches!(
            failed.outcome,
            Outcome::Failed {
                error: Failure {
                    kind: FailureKind::Admission,
                    ..
                }
            }
        ));
    }
    assert!(matches!(
        report.decimals.outcome,
        Outcome::Read { value: 255 }
    ));
}
