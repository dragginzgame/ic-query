use super::*;
use crate::tests::{self as wire, Fixture, encode_fixture as encode_one};
use ic_platform_types::Principal;
use std::{cell::RefCell, collections::VecDeque};

fn observation(bytes: Vec<u8>) -> Observation {
    Observation {
        bytes,
        observed_at: 123,
    }
}

fn encoded<T: Fixture>(value: &T) -> Vec<u8> {
    encode_one(value, 2 * 1024 * 1024).unwrap()
}

fn row(class: NodeType, provider: Option<Principal>, dc: Option<&str>) -> wire::MarketplaceEntry {
    let mut key = provider.map_or_else(String::new, |id| format!("{id},"));
    key.push_str(class_key(&class));
    if let Some(dc) = dc {
        key.push(',');
        key.push_str(dc);
    }
    wire::MarketplaceEntry {
        key,
        entry: wire::PriceEntry {
            node_type: serde_json::from_value(serde_json::to_value(class).unwrap()).unwrap(),
            provider,
            dc: dc.map(str::to_owned),
            updated_at: i64::MIN,
            price: wire::Price {
                net_cycles: 1_000_000u64.into(),
                gross_cycles: 1_250_000u64.into(),
            },
        },
    }
}

fn project(rows: &[wire::MarketplaceEntry]) -> Result<PriceObservation, PriceError> {
    project_prices(&observation(encoded(&rows.to_vec())))
}

fn hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn independently_authored_wire_fixture_preserves_daily_amounts_and_signed_time() {
    // Literal authored from the selected live IDL, independently of Platform's
    // DTO encoder: all six class alternatives, one type4_2 row, -1 timestamp.
    let frame = hex(
        "4449444c076d016c029f93c60271f2f0998407026c05ffae0103b7fff5810174d7fa89890104b1e2e7990205899dadc40c066e716b06bae0add6017fac8bcfc50f7fad8bcfc50f7fae8bcfc50f7faf8bcfc50f7fb08bcfc50f7f6e686c02a8db82e4097def9790a90b7d0100010774797065342e3200ffffffffffffffff0200d0a54cc0843d",
    );
    let report = project_prices(&observation(frame.clone())).unwrap();
    let price = &report.entries[0];
    assert_eq!(price.key, "type4.2");
    assert_eq!(price.node_type, NodeType::Type4_2);
    assert_eq!(price.net_cycles_per_node_per_day, "1000000");
    assert_eq!(price.gross_cycles_per_node_per_day, "1250000");
    assert_eq!(price.updated_at_unix_nanoseconds, -1);
    assert!(price.provider.is_none() && price.data_center.is_none());
    assert_eq!(report.evidence.source_canister, CANISTER);
    assert!(report.evidence.query_signature_verified && !report.evidence.result_certified);
    let fee = project_fee(&observation(hex("4449444c000172000000000000d03f"))).unwrap();
    assert_eq!(fee.0, 0.25);
    let mut trailing = frame.clone();
    trailing.push(0);
    assert!(matches!(
        project_prices(&observation(trailing)),
        Err(PriceError::Decode(decoder::Error::Trailing))
    ));
    assert!(matches!(
        project_prices(&observation(frame[..frame.len() - 1].to_vec())),
        Err(PriceError::Decode(_))
    ));
}

#[test]
fn classes_scopes_raw_text_and_exact_numeric_extremes() {
    let provider = Some(Principal::from_slice(&[1]));
    for class in [
        NodeType::Type4,
        NodeType::Type4_1,
        NodeType::Type4_2,
        NodeType::Type4_3,
        NodeType::Type4_4,
        NodeType::Type4_5,
    ] {
        for provider in [None, provider] {
            for dc in [None, Some(""), Some(" MiXeD\nDC ")] {
                let mut entry = row(class, provider, dc);
                entry.entry.price.net_cycles = "9".repeat(256).parse().unwrap();
                entry.entry.price.gross_cycles = entry.entry.price.net_cycles.clone();
                let report = project(&[entry.clone()]).unwrap();
                let got = &report.entries[0];
                assert_eq!(got.node_type, class);
                assert_eq!(got.provider, provider);
                assert_eq!(got.data_center.as_deref(), dc);
                assert_eq!(got.net_cycles_per_node_per_day, "9".repeat(256));
                assert_eq!(got.updated_at_unix_nanoseconds, i64::MIN);
                let json = serde_json::to_value(report).unwrap();
                assert_eq!(
                    json["entries"][0]["net_cycles_per_node_per_day"],
                    "9".repeat(256)
                );
                entry.entry.updated_at = i64::MAX;
                entry.entry.price.net_cycles = 0u64.into();
                assert_eq!(
                    project(&[entry]).unwrap().entries[0].updated_at_unix_nanoseconds,
                    i64::MAX
                );
            }
        }
    }
}

#[test]
fn identity_and_amount_refusals_are_typed() {
    let good = row(NodeType::Type4_1, None, None);
    assert!(matches!(
        project(&[good.clone(), good.clone()]),
        Err(PriceError::Admission(AdmissionError::DuplicateKey))
    ));
    for key in ["type4_1".to_owned(), "wrong".to_owned(), "x".repeat(512)] {
        let mut bad = good.clone();
        bad.key = key;
        assert!(matches!(
            project(&[bad]),
            Err(PriceError::Admission(AdmissionError::KeyMismatch))
        ));
    }
    let mut bad = good.clone();
    bad.entry.price.gross_cycles = 999u64.into();
    assert!(matches!(
        project(&[bad]),
        Err(PriceError::Admission(AdmissionError::GrossBelowNet))
    ));
    let mut bad = good;
    bad.entry.price.gross_cycles = format!("1{}", "0".repeat(256)).parse().unwrap();
    assert!(matches!(
        project(&[bad]),
        Err(PriceError::Admission(AdmissionError::CycleDigits))
    ));
}

#[test]
fn bounds_empty_lists_and_worst_case_supported_collection() {
    assert!(project(&[]).unwrap().entries.is_empty());
    let rows: Vec<_> = (0..MAX_ROWS)
        .map(|n| row(NodeType::Type4, None, Some(&n.to_string())))
        .collect();
    assert_eq!(project(&rows).unwrap().entries.len(), MAX_ROWS);
    let mut excess = rows.clone();
    excess.push(row(NodeType::Type4, None, Some("extra")));
    assert!(matches!(
        project(&excess),
        Err(PriceError::Admission(AdmissionError::Limit {
            field: "rows",
            actual: 1001,
            ..
        }))
    ));
    for (len, allowed) in [(256, true), (258, false)] {
        let dc = "é".repeat(len / 2);
        assert_eq!(
            project(&[row(NodeType::Type4, None, Some(&dc))]).is_ok(),
            allowed
        );
    }
    let mut bad = row(NodeType::Type4, None, None);
    bad.key = "x".repeat(513);
    assert!(matches!(
        project(&[bad]),
        Err(PriceError::Admission(AdmissionError::Limit {
            field: "key_bytes",
            ..
        }))
    ));
    // Aggregate decoder bounds still apply when each independent field is legal.
    let mut heavy = rows;
    for row in &mut heavy {
        row.entry.price.gross_cycles = "9".repeat(256).parse().unwrap();
        row.entry.price.net_cycles = row.entry.price.gross_cycles.clone();
    }
    assert_eq!(project(&heavy).unwrap().entries.len(), MAX_ROWS);
    assert!(matches!(
        project_prices(&observation(vec![0; LIMITS.bytes + 1])),
        Err(PriceError::Decode(decoder::Error::Limit))
    ));
    let json = serde_json::to_value(PriceError::Admission(AdmissionError::Limit {
        field: "rows",
        actual: 1001,
        maximum: 1000,
    }))
    .unwrap();
    assert_eq!(json["admission"]["actual"], 1001);
}

#[test]
fn protocol_vector_limit_refuses_before_report_admission() {
    let rows = vec![row(NodeType::Type4, None, None); LIMITS.vector + 1];
    assert!(matches!(
        project(&rows),
        Err(PriceError::Decode(decoder::Error::Limit))
    ));
}

#[test]
fn fee_admission_does_not_recompute_gross_or_serialize_nonfinite_values() {
    for fee in [0.0, -0.0, 0.25, 1.0, f64::MAX] {
        let (actual, _) = project_fee(&observation(encoded(&fee))).unwrap();
        assert_eq!(actual.to_bits(), fee.to_bits());
    }
    for fee in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1] {
        assert!(matches!(
            project_fee(&observation(encoded(&fee))),
            Err(PriceError::Admission(AdmissionError::InvalidFee))
        ));
    }
}

struct Step {
    wait: u64,
    result: Result<Vec<u8>, PriceError>,
}

struct Fake {
    calls: RefCell<Vec<&'static str>>,
    steps: RefCell<VecDeque<Step>>,
}

impl Fake {
    fn new(steps: Vec<Step>) -> Self {
        Self {
            calls: RefCell::new(vec![]),
            steps: RefCell::new(steps.into()),
        }
    }
}

impl Source for Fake {
    async fn query(&self, method: &'static str) -> Result<Observation, PriceError> {
        self.calls.borrow_mut().push(method);
        let step = self.steps.borrow_mut().pop_front().unwrap();
        tokio::time::sleep(Duration::from_secs(step.wait)).await;
        step.result.map(observation)
    }
}

fn price_step(wait: u64) -> Step {
    Step {
        wait,
        result: Ok(encoded(&vec![row(NodeType::Type4, None, None)])),
    }
}

fn fee_step(wait: u64, fee: f64) -> Step {
    Step {
        wait,
        result: Ok(encoded(&fee)),
    }
}

#[tokio::test(start_paused = true)]
async fn two_reads_keep_distinct_evidence_and_later_failures_keep_prices() {
    for fee in [
        fee_step(1, 0.5),
        fee_step(1, f64::NAN),
        Step {
            wait: 1,
            result: Err(crate::ReadError::Deadline.into()),
        },
        Step {
            wait: 1,
            result: Ok(vec![0]),
        },
    ] {
        let source = Fake::new(vec![price_step(1), fee]);
        let report = collect(&source).await.unwrap();
        assert_eq!(
            source.calls.borrow().as_slice(),
            [PRICES_METHOD, FEE_METHOD]
        );
        assert_eq!(
            report.prices.entries[0].gross_cycles_per_node_per_day,
            "1250000"
        );
        assert!(!report.atomic_snapshot);
        assert_eq!(report.elapsed, Duration::from_secs(2));
        match &report.network_fee {
            FeeOutcome::Read { markup, evidence } => {
                assert!(report.complete);
                assert_eq!(*markup, 0.5);
                assert_eq!(evidence.source_method, FEE_METHOD);
            }
            FeeOutcome::Failed { .. } => assert!(!report.complete),
            FeeOutcome::OverallDeadline => panic!("unexpected overall timeout"),
        }
        assert_eq!(report.prices.evidence.source_method, PRICES_METHOD);
        serde_json::to_value(&report).unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn initial_failure_never_starts_fee_and_deadline_spans_both_reads() {
    for step in [
        Step {
            wait: 0,
            result: Err(crate::ReadError::Deadline.into()),
        },
        Step {
            wait: 0,
            result: Ok(vec![0]),
        },
        price_step(31),
    ] {
        let source = Fake::new(vec![step]);
        assert!(collect(&source).await.is_err());
        assert_eq!(source.calls.borrow().as_slice(), [PRICES_METHOD]);
    }
    let source = Fake::new(vec![price_step(19), fee_step(19, 0.25)]);
    let report = collect(&source).await.unwrap();
    assert!(!report.complete);
    assert_eq!(report.elapsed, OVERALL);
    assert_eq!(report.prices.entries.len(), 1);
    assert!(matches!(report.network_fee, FeeOutcome::OverallDeadline));
}

#[tokio::test(start_paused = true)]
async fn cancellation_drops_collection_without_later_queries() {
    for seconds in [1, 3] {
        let source = Fake::new(vec![price_step(2), fee_step(20, 0.25)]);
        {
            let future = collect(&source);
            tokio::pin!(future);
            tokio::select! { _ = &mut future => panic!("should remain pending"), _ = tokio::time::sleep(Duration::from_secs(seconds)) => {} }
        }
        let calls = source.calls.borrow().len();
        assert_eq!(calls, if seconds == 1 { 1 } else { 2 });
        tokio::time::advance(Duration::from_secs(60)).await;
        assert_eq!(source.calls.borrow().len(), calls);
    }
}
