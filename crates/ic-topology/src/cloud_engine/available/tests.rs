use super::*;
use ic_platform::candid_codec::encode_one;
use std::{cell::RefCell, collections::VecDeque};

fn id(n: u16) -> Principal {
    Principal::from_slice(&n.to_le_bytes())
}
fn encoded<T: ic_platform::candid_codec::Encode>(value: &T) -> Vec<u8> {
    encode_one(value, 1024 * 1024).unwrap()
}
fn hex(raw: &str) -> Vec<u8> {
    raw.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn observation(bytes: Vec<u8>) -> Observation {
    Observation {
        bytes,
        observed_at: 123,
    }
}
fn node(n: u16) -> wire::AvailableNode {
    wire::AvailableNode {
        node_id: id(n),
        node_type: wire::NodeType::Type4_5,
        operator: id(6000),
        provider: id(7000),
        dc: Some(" MiXeD\nDC ".into()),
        sev_enabled: true,
    }
}
fn page(
    nodes: Vec<wire::AvailableNode>,
    next: Option<Principal>,
    version: u64,
) -> wire::AvailableNodesPage {
    wire::AvailableNodesPage {
        nodes,
        next,
        registry_version: version,
    }
}
fn response(page: wire::AvailableNodesPage) -> Vec<u8> {
    encoded(&wire::AvailableNodesPageResult::Ok(Some(page)))
}

const PAGE_HEX: &str = "4449444c096b029cc20101e58eb402086e026c03f3898ac80403d6eef5a00678b1f0fafd09046e686d056c07ffae0106d7fa89890107b8aaccb70168b1e2e7990268a4b7cca30468a6fbeca30b7e9cccb3f70f716e716b06bae0add6017fac8bcfc50f7fad8bcfc50f7fae8bcfc50f7faf8bcfc50f7fb08bcfc50f7f6b01cfa0def2067f0100000100ffffffffffffffff01010005010101010102010103010b323030313a6462383a3a31";

#[test]
fn independent_literal_preserves_full_version_raw_flags_and_skips_ip() {
    // Independently authored from selected live IDL, including the unselected IP
    // field, all six class tags, full u64 version and present-empty DC.
    let decoded = decode_page(&observation(hex(PAGE_HEX))).unwrap();
    assert_eq!(decoded.registry_version, u64::MAX);
    assert!(decoded.next.is_none());
    assert_eq!(decoded.nodes.len(), 1);
    assert_eq!(decoded.nodes[0].node_id, Principal::from_slice(&[1]));
    assert_eq!(decoded.nodes[0].node_type, wire::NodeType::Type4_5);
    assert_eq!(decoded.nodes[0].dc.as_deref(), Some(""));
    assert!(decoded.nodes[0].sev_enabled);
    let json = serde_json::to_string(&decoded).unwrap();
    assert!(!json.contains("ip_address") && !json.contains("2001:db8"));
    let unknown = PAGE_HEX.replacen("b08bcfc50f", "b18bcfc50f", 1);
    assert!(matches!(
        decode_page(&observation(hex(&unknown))),
        Err(AvailableError::Decode(decoder::Error::Type))
    ));
    let mut trailing = hex(PAGE_HEX);
    trailing.push(0);
    assert!(matches!(
        decode_page(&observation(trailing)),
        Err(AvailableError::Decode(decoder::Error::Trailing))
    ));
    assert!(matches!(
        decode_page(&observation(vec![0; LIMITS.bytes + 1])),
        Err(AvailableError::Decode(decoder::Error::Limit))
    ));
}

#[test]
fn request_is_exactly_public_rejoining_none_with_100_node_limit_and_cursor() {
    assert_eq!(
        source::page_argument(Some(Principal::from_slice(&[1]))).unwrap(),
        hex("4449444c046c03bbbe84a80701e2e8ada00802e5df81c00d036e7d6e686e71010001640101010100")
    );
    for cursor in [None, Some(id(1)), Some(Principal::from_slice(&[255; 29]))] {
        let bytes = source::page_argument(cursor).unwrap();
        let decoded: wire::ListAvailableNodesPageArgs =
            decoder::decode_one(&bytes, LIMITS).unwrap();
        assert!(decoded.rejoining.is_none());
        assert_eq!(decoded.start, cursor);
        assert_eq!(decoded.limit, Some(100u64.into()));
    }
}

#[test]
fn skipped_ip_work_is_bounded_independently_of_selected_fields() {
    // IP is the last field in the literal. Replace its 11-byte string with
    // literal ULEB lengths for 64 KiB and 128 KiB, both below the response cap.
    for (length, prefix, accepted) in [
        (64 * 1024, [0x80, 0x80, 0x04], true),
        (128 * 1024, [0x80, 0x80, 0x08], false),
    ] {
        let mut frame = hex(PAGE_HEX);
        frame.truncate(frame.len() - 12);
        frame.extend(prefix);
        frame.extend(std::iter::repeat_n(b'x', length));
        let result = decode_page(&observation(frame));
        if accepted {
            assert_eq!(result.unwrap().nodes.len(), 1);
        } else {
            assert!(matches!(
                result,
                Err(AvailableError::Decode(decoder::Error::Limit))
            ));
        }
    }
}

struct Step {
    wait: u64,
    result: Result<Vec<u8>, AvailableError>,
}
struct Fake {
    status: RefCell<Option<Step>>,
    steps: RefCell<VecDeque<Step>>,
    calls: RefCell<Vec<Option<Principal>>>,
}
impl Fake {
    fn new(steps: Vec<Step>) -> Self {
        Self {
            status: RefCell::new(Some(Step {
                wait: 0,
                result: Ok(encoded(&wire::RegistryCopyStatus {
                    version: 10,
                    latest: 20,
                    caught_up: false,
                })),
            })),
            steps: RefCell::new(steps.into()),
            calls: RefCell::new(vec![]),
        }
    }
}
async fn perform(step: Step) -> Result<Observation, AvailableError> {
    tokio::time::sleep(Duration::from_secs(step.wait)).await;
    step.result.map(observation)
}
impl Source for Fake {
    async fn status(&self) -> Result<Observation, AvailableError> {
        let step = self.status.borrow_mut().take().expect("status read once");
        perform(step).await
    }
    async fn page(&self, start: Option<Principal>) -> Result<Observation, AvailableError> {
        self.calls.borrow_mut().push(start);
        let step = self
            .steps
            .borrow_mut()
            .pop_front()
            .expect("no unexpected page");
        perform(step).await
    }
}
fn good(nodes: Vec<wire::AvailableNode>, next: Option<Principal>, version: u64) -> Step {
    Step {
        wait: 0,
        result: Ok(response(page(nodes, next, version))),
    }
}
fn fail(error: AvailableError) -> Step {
    Step {
        wait: 0,
        result: Err(error),
    }
}

#[tokio::test(start_paused = true)]
async fn traversal_preserves_pages_order_and_independent_copy_status() {
    let source = Fake::new(vec![
        good(vec![node(90), node(2)], Some(id(8)), u64::MAX),
        good(vec![node(8)], None, u64::MAX),
    ]);
    let report = collect(&source, DEFAULT_PAGE_LIMIT).await.unwrap();
    assert!(report.pagination_exhausted);
    assert!(
        !report.atomic_snapshot
            && !report.point_in_time_inventory_complete
            && !report.result_certified
    );
    assert_eq!(source.calls.borrow().as_slice(), [None, Some(id(8))]);
    assert_eq!(report.registry_copy.version, 10);
    assert_eq!(report.registry_copy.latest, 20);
    assert!(!report.registry_copy.caught_up); // Status does not gate a later fresh page.
    assert_eq!(report.pages[0].nodes[0].node_id, id(90)); // No assumed principal order.
    assert_eq!(report.pages[1].start, Some(id(8)));
    assert_eq!(report.pages[1].registry_version, u64::MAX);
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["pages"][0]["nodes"][0]["data_center"], " MiXeD\nDC ");
    assert_eq!(json["pages"][0]["nodes"][0]["sev_enabled"], true);
    assert!(json["pages"][0]["nodes"][0].get("ip_address").is_none());
}

#[tokio::test(start_paused = true)]
async fn unready_and_application_errors_are_not_empty_successes() {
    let empty = collect(&Fake::new(vec![good(vec![], None, 0)]), DEFAULT_PAGE_LIMIT)
        .await
        .unwrap();
    assert!(empty.pagination_exhausted && empty.pages.len() == 1);
    let unready = Fake::new(vec![Step {
        wait: 0,
        result: Ok(encoded(&wire::AvailableNodesPageResult::Ok(None))),
    }]);
    let report = collect(&unready, DEFAULT_PAGE_LIMIT).await.unwrap();
    assert!(!report.pagination_exhausted && report.pages.is_empty());
    assert!(matches!(
        report.stop,
        TraversalStop::Failed {
            error: AvailableError::CopyNotReady,
            observed_at_unix_seconds: Some(123),
            ..
        }
    ));
    for error in [
        wire::EngineError::NotFound,
        wire::EngineError::Unauthorized,
        wire::EngineError::NotVetted,
        wire::EngineError::AlreadyExists,
        wire::EngineError::SlugConflict("raw\x1b\n".into()),
        wire::EngineError::BadRequest("raw\x1b\n".into()),
        wire::EngineError::ControllerError("raw\x1b\n".into()),
        wire::EngineError::InvalidTransition("raw\x1b\n".into()),
        wire::EngineError::Internal("raw\x1b\n".into()),
    ] {
        let source = Fake::new(vec![
            good(vec![node(1)], Some(id(2)), 5),
            Step {
                wait: 0,
                result: Ok(encoded(&wire::AvailableNodesPageResult::Err(error.clone()))),
            },
        ]);
        let report = collect(&source, DEFAULT_PAGE_LIMIT).await.unwrap();
        assert!(!report.pagination_exhausted && report.pages.len() == 1);
        if let TraversalStop::Failed {
            error: AvailableError::Application(got),
            ..
        } = report.stop
        {
            assert_eq!(got, error);
            let wrapped = AvailableError::Application(got);
            assert!(!wrapped.to_string().contains('\x1b'));
            assert_eq!(
                serde_json::to_value(&wrapped).unwrap()["application"],
                serde_json::to_value(error).unwrap()
            );
        } else {
            panic!("application cause lost");
        }
    }
}

#[tokio::test(start_paused = true)]
async fn changed_versions_and_duplicate_pages_preserve_only_admitted_prefix() {
    for (before, after) in [(u64::MAX, 0), (0, u64::MAX)] {
        let source = Fake::new(vec![
            good(vec![node(1)], Some(id(2)), before),
            good(vec![node(2)], None, after),
        ]);
        let report = collect(&source, DEFAULT_PAGE_LIMIT).await.unwrap();
        assert_eq!(report.pages.len(), 1);
        assert!(
            matches!(report.stop, TraversalStop::Failed { observed_registry_version: Some(value), error: AvailableError::Admission(PageRefusal::VersionChanged { expected, observed }), .. } if value == after && expected == before && observed == after)
        );
    }
    for nodes in [vec![node(2), node(1)], vec![node(2), node(2)]] {
        let source = Fake::new(vec![
            good(vec![node(1)], Some(id(2)), 5),
            good(nodes, None, 5),
        ]);
        let report = collect(&source, DEFAULT_PAGE_LIMIT).await.unwrap();
        assert_eq!(report.pages.len(), 1); // No row from the rejected page escapes.
        assert!(matches!(
            report.stop,
            TraversalStop::Failed {
                error: AvailableError::Admission(PageRefusal::DuplicateNode { .. }),
                ..
            }
        ));
    }
}

#[tokio::test(start_paused = true)]
async fn cursor_cycles_and_traversal_limits_make_no_extra_requests() {
    for cursors in [vec![1, 1], vec![1, 2, 1]] {
        let steps = cursors
            .iter()
            .enumerate()
            .map(|(n, cursor)| good(vec![node(n as u16 + 10)], Some(id(*cursor)), 5))
            .collect();
        let source = Fake::new(steps);
        let report = collect(&source, DEFAULT_PAGE_LIMIT).await.unwrap();
        assert_eq!(report.pages.len(), cursors.len() - 1);
        assert_eq!(source.calls.borrow().len(), cursors.len());
        assert!(matches!(
            report.stop,
            TraversalStop::Failed {
                error: AvailableError::Admission(PageRefusal::RepeatedCursor { .. }),
                ..
            }
        ));
    }
    let source = Fake::new((1..=10).map(|n| good(vec![], Some(id(n)), 5)).collect());
    let report = collect(&source, DEFAULT_PAGE_LIMIT).await.unwrap();
    assert_eq!(report.pages.len(), usize::from(DEFAULT_PAGE_LIMIT));
    assert!(matches!(report.stop, TraversalStop::PageLimit));
    assert_eq!(report.pages.last().unwrap().next, Some(id(10)));
    for exhausted in [true, false] {
        let source = Fake::new(
            (0..10)
                .map(|p| {
                    good(
                        (1..=100).map(|n| node(p * 100 + n)).collect(),
                        if p == 9 && exhausted {
                            None
                        } else {
                            Some(id((p + 1) * 100 + 1))
                        },
                        5,
                    )
                })
                .collect(),
        );
        let report = collect(&source, DEFAULT_PAGE_LIMIT).await.unwrap();
        assert_eq!(
            report.pages.iter().map(|p| p.nodes.len()).sum::<usize>(),
            PAGE_SIZE * usize::from(DEFAULT_PAGE_LIMIT)
        );
        assert_eq!(source.calls.borrow().len(), usize::from(DEFAULT_PAGE_LIMIT));
        assert_eq!(report.pagination_exhausted, exhausted);
        assert!(if exhausted {
            matches!(report.stop, TraversalStop::Exhausted)
        } else {
            matches!(report.stop, TraversalStop::NodeLimit)
        });
    }
}

#[test]
fn page_size_identity_utf8_and_full_width_admission() {
    let seen = BTreeSet::new();
    let cursors = BTreeSet::new();
    let mut nodes: Vec<_> = (0..100).map(node).collect();
    for node in &mut nodes {
        node.dc = Some("é".repeat(128));
    }
    let full = page(nodes.clone(), None, u64::MAX);
    let decoded = decode_page(&observation(response(full))).unwrap();
    admit(
        &decoded,
        Some(u64::MAX),
        &seen,
        &cursors,
        PAGE_SIZE * usize::from(DEFAULT_PAGE_LIMIT),
    )
    .unwrap();
    nodes.push(node(101));
    assert!(matches!(
        admit(
            &page(nodes, None, 1),
            None,
            &seen,
            &cursors,
            PAGE_SIZE * usize::from(DEFAULT_PAGE_LIMIT)
        ),
        Err(PageRefusal::Limit {
            field: "page_nodes",
            actual: 101,
            ..
        })
    ));
    let mut bad = node(1);
    bad.dc = Some("é".repeat(129));
    assert!(matches!(
        admit(
            &page(vec![bad], None, 1),
            None,
            &seen,
            &cursors,
            PAGE_SIZE * usize::from(DEFAULT_PAGE_LIMIT)
        ),
        Err(PageRefusal::DataCenter { bytes: 258, .. })
    ));
    for field in ["node", "operator", "provider"] {
        let mut bad = node(1);
        match field {
            "node" => bad.node_id = Principal::management_canister(),
            "operator" => bad.operator = Principal::management_canister(),
            _ => bad.provider = Principal::management_canister(),
        }
        assert!(
            matches!(admit(&page(vec![bad], None, 1), None, &seen, &cursors, PAGE_SIZE * usize::from(DEFAULT_PAGE_LIMIT)), Err(PageRefusal::EmptyIdentity { field: got }) if got == field)
        );
    }
}

#[tokio::test(start_paused = true)]
async fn initial_status_failure_stops_and_later_read_failures_keep_prefix() {
    for failure in [
        fail(crate::ReadError::Deadline.into()),
        Step {
            wait: 0,
            result: Ok(vec![0]),
        },
        Step {
            wait: 61,
            result: Ok(vec![]),
        },
    ] {
        let source = Fake::new(vec![]);
        *source.status.borrow_mut() = Some(failure);
        assert!(collect(&source, DEFAULT_PAGE_LIMIT).await.is_err());
        assert!(source.calls.borrow().is_empty());
    }
    for failure in [
        fail(crate::ReadError::Deadline.into()),
        Step {
            wait: 0,
            result: Ok(vec![0]),
        },
    ] {
        let source = Fake::new(vec![good(vec![node(1)], Some(id(2)), 5), failure]);
        let report = collect(&source, DEFAULT_PAGE_LIMIT).await.unwrap();
        assert_eq!(report.pages.len(), 1);
        assert!(
            matches!(report.stop, TraversalStop::Failed { start: Some(cursor), .. } if cursor == id(2))
        );
    }
}

#[tokio::test(start_paused = true)]
async fn shared_deadline_includes_status_and_prior_pages() {
    let mut steps: Vec<_> = (0..3)
        .map(|n| good(vec![node(n)], Some(id(n + 1)), 5))
        .collect();
    for step in &mut steps {
        step.wait = 19;
    }
    let source = Fake::new(steps);
    source.status.borrow_mut().as_mut().unwrap().wait = 10;
    let report = collect(&source, DEFAULT_PAGE_LIMIT).await.unwrap();
    assert_eq!(report.elapsed, OVERALL);
    assert_eq!(report.pages.len(), 2);
    assert_eq!(source.calls.borrow().len(), 3);
    assert!(
        matches!(report.stop, TraversalStop::OverallDeadline { start: Some(cursor) } if cursor == id(2))
    );
}

#[tokio::test(start_paused = true)]
async fn cancellation_during_status_or_page_starts_no_later_reads() {
    for seconds in [1, 3, 6] {
        let mut steps = vec![
            good(vec![node(1)], Some(id(2)), 5),
            good(vec![node(2)], None, 5),
        ];
        for step in &mut steps {
            step.wait = 3;
        }
        let source = Fake::new(steps);
        source.status.borrow_mut().as_mut().unwrap().wait = 2;
        {
            let future = collect(&source, DEFAULT_PAGE_LIMIT);
            tokio::pin!(future);
            tokio::select! { _ = &mut future => panic!("unexpected completion"), _ = tokio::time::sleep(Duration::from_secs(seconds)) => {} }
        }
        let count = source.calls.borrow().len();
        assert_eq!(
            count,
            if seconds == 1 {
                0
            } else if seconds == 3 {
                1
            } else {
                2
            }
        );
        tokio::time::advance(Duration::from_secs(90)).await;
        assert_eq!(source.calls.borrow().len(), count);
    }
}

#[tokio::test(start_paused = true)]
async fn selected_page_budgets_cover_more_candidates_without_unbounded_reads() {
    for page_limit in [1, 20, MAX_PAGE_LIMIT] {
        for exhausted in [true, false] {
            let steps = (0..page_limit)
                .map(|p| {
                    good(
                        (1..=100).map(|n| node(p * 100 + n)).collect(),
                        if exhausted && p + 1 == page_limit {
                            None
                        } else {
                            Some(id((p + 1) * 100 + 1))
                        },
                        5,
                    )
                })
                .collect();
            let source = Fake::new(steps);
            let report = collect(&source, page_limit).await.unwrap();
            assert_eq!(report.page_limit, page_limit);
            assert_eq!(report.node_limit, usize::from(page_limit) * PAGE_SIZE);
            assert_eq!(
                report.pages.iter().map(|p| p.nodes.len()).sum::<usize>(),
                report.node_limit
            );
            assert_eq!(source.calls.borrow().len(), usize::from(page_limit));
            assert_eq!(report.pagination_exhausted, exhausted);
            assert!(!report.point_in_time_inventory_complete);
        }
    }
}

#[tokio::test]
async fn invalid_page_budgets_are_rejected_before_network_io() {
    for requested in [0, MAX_PAGE_LIMIT + 1, u16::MAX] {
        assert!(matches!(fetch_with_page_limit(requested).await,
            Err(AvailableError::InvalidPageLimit { requested: actual }) if actual == requested));
    }
}

#[tokio::test(start_paused = true)]
async fn larger_page_budget_keeps_the_same_overall_deadline() {
    let mut steps: Vec<_> = (0..4)
        .map(|n| good(vec![node(n)], Some(id(n + 1)), 5))
        .collect();
    for step in &mut steps {
        step.wait = 19;
    }
    let source = Fake::new(steps);
    let report = collect(&source, MAX_PAGE_LIMIT).await.unwrap();
    assert_eq!(report.pages.len(), 3);
    assert_eq!(report.elapsed, OVERALL);
    assert!(matches!(report.stop, TraversalStop::OverallDeadline { .. }));
}
