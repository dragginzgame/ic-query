use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use super::*;
use source::Observation;

// Independently authored fixtures for the pinned protocol fields, not a codec.
fn varint(mut n: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    while n >= 128 {
        bytes.push(n as u8 | 128);
        n >>= 7;
    }
    bytes.push(n as u8);
    bytes
}

fn blob(tag: u8, value: &[u8]) -> Vec<u8> {
    let mut bytes = vec![tag];
    bytes.extend(varint(value.len() as u64));
    bytes.extend(value);
    bytes
}

fn response(record: &[u8], version: u64) -> Vec<u8> {
    let mut bytes = vec![16];
    bytes.extend(varint(version));
    bytes.extend(blob(26, record));
    bytes.extend([40, 42]);
    bytes
}

fn id(value: u16) -> Principal {
    Principal::from_slice(&value.to_be_bytes())
}

struct Fixture {
    version: u64,
    records: BTreeMap<String, Vec<u8>>,
    calls: RefCell<Vec<(String, u64)>>,
    delays: BTreeMap<String, Duration>,
    latest_delay: Duration,
    active: Cell<usize>,
    max_active: Cell<usize>,
}

struct Active<'a>(&'a Cell<usize>);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

impl Source for Fixture {
    async fn latest(&self) -> Result<Observation, DistributionError> {
        self.calls
            .borrow_mut()
            .push(("get_latest_version".into(), 0));
        tokio::time::sleep(self.latest_delay).await;
        let mut bytes = vec![8];
        bytes.extend(varint(self.version));
        Ok(Observation {
            bytes,
            observed_at: 101,
        })
    }

    async fn value(&self, key: &str, version: u64) -> Result<Observation, DistributionError> {
        self.calls.borrow_mut().push((key.into(), version));
        self.active.set(self.active.get() + 1);
        self.max_active
            .set(self.max_active.get().max(self.active.get()));
        let _active = Active(&self.active);
        if let Some(delay) = self.delays.get(key) {
            tokio::time::sleep(*delay).await;
        }
        let bytes = self
            .records
            .get(key)
            .ok_or_else(|| DistributionError::RegistryRejected {
                code: 1,
                reason: "missing\n\u{1b}[31m".into(),
                key: key.as_bytes().to_vec(),
            })?
            .clone();
        Ok(Observation {
            bytes,
            observed_at: 102,
        })
    }
}

impl Fixture {
    fn new(count: u16) -> Self {
        let mut record = Vec::new();
        for n in 1..=count {
            record.extend(blob(26, id(n).as_slice()));
        }
        let mut result = Self {
            version: 300,
            records: BTreeMap::new(),
            calls: RefCell::new(Vec::new()),
            delays: BTreeMap::new(),
            latest_delay: Duration::ZERO,
            active: Cell::new(0),
            max_active: Cell::new(0),
        };
        result
            .records
            .insert(format!("subnet_record_{}", id(0)), response(&record, 200));
        for n in 1..=count {
            result.node(n, 500);
        }
        result.operator(500, &[3], "DC1");
        result.dc("dc1", "Dc1", "CH,Zurich", "Owner");
        result
    }

    fn node(&mut self, node: u16, operator: u16) {
        self.records.insert(
            format!("node_record_{}", id(node)),
            response(&blob(122, id(operator).as_slice()), 201),
        );
    }

    fn operator(&mut self, operator: u16, provider: &[u8], dc: &str) {
        let mut bytes = blob(10, id(operator).as_slice());
        bytes.extend(blob(26, provider));
        bytes.extend(blob(34, dc.as_bytes()));
        self.records.insert(
            format!("node_operator_record_{}", id(operator)),
            response(&bytes, 202),
        );
    }

    fn dc(&mut self, key: &str, returned: &str, region: &str, owner: &str) {
        let mut bytes = blob(10, returned.as_bytes());
        bytes.extend(blob(18, region.as_bytes()));
        bytes.extend(blob(26, owner.as_bytes()));
        self.records
            .insert(format!("data_center_record_{key}"), response(&bytes, 203));
    }
}

#[tokio::test(start_paused = true)]
async fn pins_full_width_version_deduplicates_relations_and_counts_nodes_once() {
    let mut source = Fixture::new(5);
    source.version = u64::MAX;
    source.node(5, 501);
    source.operator(501, &[3], "dc1");
    for n in 1..=5 {
        source
            .delays
            .insert(format!("node_record_{}", id(n)), Duration::from_secs(1));
    }
    let report = collect(id(0), 100, &source).await.unwrap();
    assert!(report.complete && !report.overall_deadline_reached);
    assert_eq!(report.operators.len(), 2);
    assert_eq!(report.data_centers.len(), 1);
    assert_eq!(report.summary.providers.len(), 1);
    assert_eq!(report.summary.providers[0].nodes, 5);
    assert_eq!(report.summary.data_centers[0].nodes, 5);
    assert_eq!(report.summary.regions[0].nodes, 5);
    assert_eq!(report.summary.owners[0].nodes, 5);
    assert_eq!(source.max_active.get(), 4);
    let calls = source.calls.borrow();
    assert_eq!(calls.len(), 10); // version + subnet + five nodes + two operators + one DC
    assert!(calls[1..].iter().all(|(_, version)| *version == u64::MAX));
    let json = serde_json::to_value(report).unwrap();
    assert_eq!(json["requested_registry_version"], u64::MAX);
    assert_eq!(json["result_certified"], false);
    assert_eq!(
        json["operators"][0]["outcome"]["report"]["data_center"]["raw"],
        "DC1"
    );
    assert_eq!(json["data_centers"][0]["outcome"]["report"]["id"], "Dc1");
    assert_eq!(
        json["data_centers"][0]["outcome"]["report"]["evidence"]["record_version"],
        203
    );
}

#[tokio::test(start_paused = true)]
async fn zero_and_excess_membership_refuse_before_fanout_and_exact_limit_is_retained() {
    let mut source = Fixture::new(1);
    source.version = 0;
    assert!(matches!(
        collect(id(0), 0, &source).await,
        Err(DistributionError::ZeroVersion)
    ));
    assert_eq!(source.calls.borrow().len(), 1);
    let source = Fixture::new(129);
    assert!(matches!(
        collect(id(0), 0, &source).await,
        Err(DistributionError::NodeLimit)
    ));
    assert_eq!(source.calls.borrow().len(), 2);
    let source = Fixture::new(128);
    let report = collect(id(0), 0, &source).await.unwrap();
    assert_eq!(report.nodes.len(), 128);
    assert_eq!(report.summary.providers[0].nodes, 128);
    assert!(report.complete);
    let source = Fixture::new(0);
    assert!(collect(id(0), 0, &source).await.unwrap().complete);
    assert_eq!(source.calls.borrow().len(), 2);
}

#[tokio::test(start_paused = true)]
async fn version_skew_refuses_initial_and_leaf_records_without_latest_fallback() {
    let mut source = Fixture::new(1);
    source.version = 199;
    assert!(matches!(
        collect(id(0), 0, &source).await,
        Err(DistributionError::VersionSkew {
            requested: 199,
            returned: 200
        })
    ));
    source.version = 200;
    source.calls.borrow_mut().clear();
    let report = collect(id(0), 0, &source).await.unwrap();
    assert!(!report.complete);
    assert!(matches!(
        report.nodes[0].outcome,
        RecordOutcome::Failed {
            error: DistributionError::VersionSkew {
                requested: 200,
                returned: 201
            }
        }
    ));
    assert_eq!(report.summary.provider_unknown, 1);
    assert!(report.operators.is_empty());
    assert_eq!(source.calls.borrow().len(), 3);
}

#[tokio::test(start_paused = true)]
async fn provider_and_location_failures_have_independent_coverage() {
    let mut source = Fixture::new(4);
    source.operator(500, &[3], "missing");
    source.node(2, 501);
    source.operator(501, &[], "dc1");
    source.node(3, 502);
    source.operator(502, &[4], " dc1 ");
    source.node(4, 503); // Unavailable operator: neither relationship can resolve.
    let report = collect(id(0), 0, &source).await.unwrap();
    assert_eq!(
        (
            report.summary.provider_resolved,
            report.summary.provider_unknown
        ),
        (2, 2)
    );
    assert_eq!(
        (
            report.summary.location_resolved,
            report.summary.location_unknown
        ),
        (1, 3)
    );
    assert_eq!(report.summary.data_center_references_resolved, 2);
    assert_eq!(report.summary.selected_nodes, 4);
    assert!(!report.complete);
    assert!(matches!(
        report.relationships[0].location,
        Relationship::Unavailable {
            reason: UnavailableReason::DataCenterRecord
        }
    ));
    assert!(matches!(
        report.relationships[1].provider,
        Relationship::Unavailable {
            reason: UnavailableReason::MissingReference
        }
    ));
    assert!(matches!(
        report.relationships[2].data_center_reference,
        Relationship::Unavailable {
            reason: UnavailableReason::InvalidReference
        }
    ));
    assert!(matches!(
        report.relationships[3].provider,
        Relationship::Unavailable {
            reason: UnavailableReason::OperatorRecord
        }
    ));
    let json = serde_json::to_value(report).unwrap();
    assert_eq!(
        json["operators"][3]["outcome"]["error"]["reason"],
        "missing\n\u{1b}[31m"
    );
}

#[tokio::test(start_paused = true)]
async fn absent_location_differs_from_invalid_and_oversized_values_remain_bounded() {
    let mut source = Fixture::new(2);
    source.operator(500, &[3], "");
    source.node(2, 501);
    source.operator(501, &[0; 30], &"x".repeat(257));
    let report = collect(id(0), 0, &source).await.unwrap();
    assert!(report.data_centers.is_empty());
    assert!(matches!(
        report.relationships[0].location,
        Relationship::Unavailable {
            reason: UnavailableReason::MissingReference
        }
    ));
    assert!(matches!(
        report.relationships[1].location,
        Relationship::Unavailable {
            reason: UnavailableReason::InvalidReference
        }
    ));
    let operator = report.operators[1].outcome.report().unwrap();
    assert!(operator.provider.raw.is_none() && operator.provider.identity.is_none());
    assert_eq!(operator.provider.byte_length, 30);
    assert!(operator.data_center.raw.is_none() && operator.data_center.identity.is_none());
    assert_eq!(operator.data_center.byte_length, 257);
    assert_eq!(report.summary.provider_resolved, 1);
}

#[test]
fn key_admission_checks_utf8_before_and_after_lowercasing_without_trimming() {
    assert_eq!(source::data_center_identity("DC1").as_deref(), Some("dc1"));
    assert_eq!(
        source::data_center_identity(&"A".repeat(256)),
        Some("a".repeat(256))
    );
    for text in ["", " dc1", "dc1\n", &"x".repeat(257), &"İ".repeat(128)] {
        assert!(source::data_center_identity(text).is_none());
    }
}

#[tokio::test(start_paused = true)]
async fn operator_and_data_center_identity_mismatches_refuse_joins() {
    let mut source = Fixture::new(2);
    source.node(2, 501);
    source.records.insert(
        format!("node_operator_record_{}", id(501)),
        response(&blob(10, id(999).as_slice()), 200),
    );
    source.dc("dc1", "other", "region", "owner");
    let report = collect(id(0), 0, &source).await.unwrap();
    assert!(matches!(
        report.operators[1].outcome,
        RecordOutcome::Failed {
            error: DistributionError::IdentityMismatch
        }
    ));
    assert!(matches!(
        report.data_centers[0].outcome,
        RecordOutcome::Failed {
            error: DistributionError::IdentityMismatch
        }
    ));
    assert_eq!(
        (
            report.summary.provider_resolved,
            report.summary.location_resolved
        ),
        (1, 0)
    );
}

#[tokio::test(start_paused = true)]
async fn empty_embedded_operator_id_is_allowed_but_details_limits_are_enforced() {
    let mut source = Fixture::new(1);
    let mut operator = blob(26, &[3]);
    operator.extend(blob(34, b"dc1"));
    source.records.insert(
        format!("node_operator_record_{}", id(500)),
        response(&operator, 202),
    );
    source.dc("dc1", "dc1", &"x".repeat(257), "owner");
    let report = collect(id(0), 0, &source).await.unwrap();
    assert!(
        report.operators[0]
            .outcome
            .report()
            .unwrap()
            .registered_operator_bytes
            .is_empty()
    );
    assert!(matches!(
        report.data_centers[0].outcome,
        RecordOutcome::Failed {
            error: DistributionError::TextLimit
        }
    ));
    assert_eq!(report.summary.provider_resolved, 1);
    source.dc("dc1", "dc1", "", "");
    let report = collect(id(0), 0, &source).await.unwrap();
    assert!(report.complete); // Record complete, declarations unspecified.
    assert_eq!(
        (report.summary.region_unknown, report.summary.owner_unknown),
        (1, 1)
    );
}

#[tokio::test(start_paused = true)]
async fn shared_deadline_retains_finished_cancelled_and_unstarted_nodes() {
    let mut source = Fixture::new(6);
    for n in 2..=4 {
        source
            .delays
            .insert(format!("node_record_{}", id(n)), Duration::from_secs(61));
    }
    let report = collect(id(0), 0, &source).await.unwrap();
    assert!(report.overall_deadline_reached && !report.complete);
    assert!(report.nodes[0].outcome.report().is_some());
    for row in &report.nodes[1..4] {
        assert!(matches!(row.outcome, RecordOutcome::OverallDeadline));
        assert!(row.started_after.is_some() && row.finished_after.is_none());
    }
    for row in &report.nodes[4..] {
        assert!(matches!(row.outcome, RecordOutcome::NotStarted));
        assert!(row.started_after.is_none());
    }
    assert!(matches!(
        report.operators[0].outcome,
        RecordOutcome::NotStarted
    ));
    assert_eq!(report.summary.provider_unknown, 6);
    assert_eq!(source.active.get(), 0); // No detached work survives cancellation.
    assert_eq!(source.calls.borrow().len(), 6);
    tokio::time::sleep(Duration::from_secs(100)).await;
    assert_eq!(source.calls.borrow().len(), 6);
}

#[tokio::test(start_paused = true)]
async fn initial_deadline_and_caller_cancellation_do_not_start_later_phases() {
    let mut source = Fixture::new(1);
    source.latest_delay = Duration::from_secs(61);
    assert!(matches!(
        collect(id(0), 0, &source).await,
        Err(DistributionError::InitialDeadline)
    ));
    assert_eq!(source.calls.borrow().len(), 1);
    source.latest_delay = Duration::ZERO;
    source.calls.borrow_mut().clear();
    source
        .delays
        .insert(format!("node_record_{}", id(1)), Duration::from_secs(10));
    assert!(
        tokio::time::timeout(Duration::from_secs(1), collect(id(0), 0, &source))
            .await
            .is_err()
    );
    assert_eq!(source.active.get(), 0);
    assert_eq!(source.calls.borrow().len(), 3);
    tokio::time::sleep(Duration::from_secs(100)).await;
    assert_eq!(source.calls.borrow().len(), 3);
}

#[test]
fn malformed_missing_and_rejected_envelopes_preserve_distinct_failures() {
    assert!(matches!(
        envelope(&[26, 255], 1),
        Err(DistributionError::Codec(_))
    ));
    assert!(matches!(
        envelope(&[], 1),
        Err(DistributionError::MissingContent)
    ));
    let mut error = vec![8, 3];
    error.extend(blob(18, b"raw\n"));
    error.extend(blob(26, &[0, 255]));
    let error = envelope(&blob(10, &error), 1).unwrap_err();
    let json = serde_json::to_value(error).unwrap();
    assert_eq!(json["kind"], "registry_rejected");
    assert_eq!(json["reason"], "raw\n");
    assert_eq!(json["key"], serde_json::json!([0, 255]));
}

#[tokio::test(start_paused = true)]
async fn exact_maximum_distinct_relationships_use_at_most_386_logical_calls() {
    let mut source = Fixture::new(128);
    for n in 1..=128 {
        source.node(n, 500 + n);
        let dc = format!("dc{n}");
        source.operator(500 + n, id(1000 + n).as_slice(), &dc);
        source.dc(&dc, &dc, "region", "owner");
    }
    let report = collect(id(0), 0, &source).await.unwrap();
    assert!(report.complete);
    assert_eq!(report.operators.len(), 128);
    assert_eq!(report.data_centers.len(), 128);
    assert_eq!(report.summary.providers.len(), 128);
    assert_eq!(report.summary.regions[0].nodes, 128);
    assert_eq!(source.calls.borrow().len(), 386);
}

#[tokio::test(start_paused = true)]
async fn relation_phases_share_the_initial_deadline_and_retain_resolved_provider() {
    let mut source = Fixture::new(1);
    source.latest_delay = Duration::from_secs(15);
    source
        .delays
        .insert(format!("subnet_record_{}", id(0)), Duration::from_secs(15));
    source
        .delays
        .insert(format!("node_record_{}", id(1)), Duration::from_secs(15));
    source.delays.insert(
        format!("node_operator_record_{}", id(500)),
        Duration::from_secs(10),
    );
    source
        .delays
        .insert("data_center_record_dc1".into(), Duration::from_secs(10));
    let report = collect(id(0), 0, &source).await.unwrap();
    assert_eq!(report.elapsed, Duration::from_secs(60));
    assert!(report.overall_deadline_reached && !report.complete);
    assert_eq!(report.summary.provider_resolved, 1);
    assert_eq!(report.summary.data_center_references_resolved, 1);
    assert_eq!(report.summary.location_unknown, 1);
    assert!(matches!(
        report.data_centers[0].outcome,
        RecordOutcome::OverallDeadline
    ));
    assert_eq!(source.active.get(), 0);
}
