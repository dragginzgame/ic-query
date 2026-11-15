use super::*;
use available::{
    AvailableError, AvailableNode, AvailableNodesReport, AvailablePage, RegistryCopyObservation,
    TraversalStop,
};
use ic_platform::cloud_engine_types::NodeType;
use prices::{
    CloudEnginePricesReport, FeeOutcome, MarketplacePrice, PriceError, PriceEvidence,
    PriceObservation,
};
use std::cell::Cell;

fn id(n: u8) -> Principal {
    Principal::from_slice(&[n])
}
fn node(dc: Option<&str>) -> AvailableNode {
    AvailableNode {
        node_id: id(1),
        node_type: NodeType::Type4,
        operator_id: id(2),
        provider_id: id(3),
        data_center: dc.map(str::to_owned),
        sev_enabled: true,
    }
}
fn price(class: NodeType, provider: Option<Principal>, dc: Option<&str>) -> MarketplacePrice {
    MarketplacePrice {
        key: format!("{provider:?},{class:?},{dc:?}"),
        node_type: class,
        provider,
        data_center: dc.map(str::to_owned),
        net_cycles_per_node_per_day: "123456789012345678901234567890".into(),
        gross_cycles_per_node_per_day: "123456789012345678901234567891".into(),
        updated_at_unix_nanoseconds: -1,
    }
}
fn availability(exhausted: bool) -> AvailableNodesReport {
    AvailableNodesReport {
        schema: 1,
        page_limit: 10,
        node_limit: 1000,
        started_at_unix_seconds: 10,
        elapsed: Duration::from_secs(1),
        endpoint: "https://ic0.app",
        source_canister: super::super::CANISTER,
        source_method: "listAvailableNodesPage",
        rejoining: None,
        query_signature_verified: true,
        result_certified: false,
        atomic_snapshot: false,
        point_in_time_inventory_complete: false,
        pagination_exhausted: exhausted,
        registry_copy: RegistryCopyObservation {
            source_method: "registryCopyStatus",
            observed_at_unix_seconds: 11,
            version: 1,
            latest: 2,
            caught_up: false,
        },
        pages: vec![AvailablePage {
            start: None,
            next: if exhausted { None } else { Some(id(1)) },
            registry_version: 2,
            observed_at_unix_seconds: 12,
            nodes: vec![node(Some("dc"))],
        }],
        stop: if exhausted {
            TraversalStop::Exhausted
        } else {
            TraversalStop::PageLimit
        },
    }
}
fn evidence(method: &'static str) -> PriceEvidence {
    PriceEvidence {
        endpoint: "https://ic0.app",
        source_canister: super::super::CANISTER,
        source_method: method,
        observed_at_unix_seconds: 20,
        query_signature_verified: true,
        result_certified: false,
    }
}
fn prices(complete: bool) -> CloudEnginePricesReport {
    CloudEnginePricesReport {
        schema: 1,
        started_at_unix_seconds: 19,
        elapsed: Duration::from_secs(1),
        complete,
        atomic_snapshot: false,
        prices: PriceObservation {
            entries: vec![price(NodeType::Type4, None, None)],
            evidence: evidence("listMarketplacePrices"),
        },
        network_fee: if complete {
            FeeOutcome::Read {
                markup: 0.25,
                evidence: evidence("getNetworkFee"),
            }
        } else {
            FeeOutcome::OverallDeadline
        },
    }
}

#[test]
fn exact_scopes_retain_all_overlaps_without_selecting_precedence() {
    let rows = vec![
        price(NodeType::Type4, None, None),
        price(NodeType::Type4, Some(id(3)), None),
        price(NodeType::Type4, None, Some("dc")),
        price(NodeType::Type4, Some(id(3)), Some("dc")),
        price(NodeType::Type4_1, None, None),
        price(NodeType::Type4, Some(id(4)), None),
        price(NodeType::Type4, None, Some("DC")),
        price(NodeType::Type4, None, Some(" dc ")),
        price(NodeType::Type4, None, Some("")),
    ];
    assert_eq!(matching_indices(&node(Some("dc")), &rows), [0, 1, 2, 3]);
    assert_eq!(matching_indices(&node(None), &rows), [0, 1]);
    assert_eq!(matching_indices(&node(Some("")), &rows), [0, 1, 8]);
    assert_eq!(matching_indices(&node(Some("DC")), &rows), [0, 1, 6]);
    for class in [
        NodeType::Type4,
        NodeType::Type4_1,
        NodeType::Type4_2,
        NodeType::Type4_3,
        NodeType::Type4_4,
        NodeType::Type4_5,
    ] {
        let mut candidate = node(None);
        candidate.node_type = class.clone();
        assert_eq!(
            matching_indices(&candidate, &[price(class, None, None)]),
            [0]
        );
    }
}

#[tokio::test]
async fn coverage_is_independent_of_collection_and_preserves_raw_evidence() {
    for exhausted in [false, true] {
        for complete in [false, true] {
            let report = collect(async { Ok(availability(exhausted)) }, || async {
                Ok(prices(complete))
            })
            .await
            .unwrap();
            assert_eq!(report.collection_complete, exhausted && complete);
            assert!(!report.atomic_snapshot);
            let json = serde_json::to_value(&report).unwrap();
            assert_eq!(
                json["pricing"]["matches"][0]["price_indices"],
                serde_json::json!([0])
            );
            assert_eq!(
                json["pricing"]["report"]["prices"]["entries"][0]["net_cycles_per_node_per_day"],
                "123456789012345678901234567890"
            );
            assert_eq!(
                json["pricing"]["report"]["prices"]["entries"][0]["updated_at_unix_nanoseconds"],
                -1
            );
            assert_eq!(report.availability.pages[0].registry_version, 2);
            assert!(!report.availability.registry_copy.caught_up);
        }
    }
    let report = collect(async { Ok(availability(true)) }, || async {
        let mut prices = prices(true);
        prices.prices.entries.clear();
        Ok(prices)
    })
    .await
    .unwrap();
    assert!(report.collection_complete);
    let CandidatePricing::Read { matches, .. } = report.pricing else {
        panic!()
    };
    assert!(matches[0].price_indices.is_empty());
}

#[tokio::test]
async fn price_failure_keeps_candidates_and_initial_failure_starts_no_pricing() {
    let report = collect(async { Ok(availability(false)) }, || async {
        Err(PriceError::OverallDeadline)
    })
    .await
    .unwrap();
    assert!(!report.collection_complete);
    assert_eq!(report.availability.pages[0].nodes[0].node_id, id(1));
    assert!(matches!(
        report.pricing,
        CandidatePricing::Failed {
            error: PriceError::OverallDeadline
        }
    ));
    let called = Cell::new(false);
    let result = collect(async { Err(AvailableError::CopyNotReady) }, || {
        called.set(true);
        async { Ok(prices(true)) }
    })
    .await;
    assert!(matches!(result, Err(AvailableError::CopyNotReady)));
    assert!(!called.get());
    for invalid in [0, 101, u16::MAX] {
        assert!(
            matches!(fetch(invalid).await, Err(AvailableError::InvalidPageLimit { requested }) if requested == invalid)
        );
    }
}

#[tokio::test(start_paused = true)]
async fn sequential_budgets_and_cancellation_do_not_start_later_work() {
    let start = Instant::now();
    let report = collect(
        async {
            tokio::time::sleep(Duration::from_secs(60)).await;
            Ok(availability(false))
        },
        || async {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Err(PriceError::OverallDeadline)
        },
    )
    .await
    .unwrap();
    assert_eq!(start.elapsed(), Duration::from_secs(90));
    assert_eq!(report.elapsed, Duration::from_secs(90));
    let called = Cell::new(false);
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        collect(std::future::pending(), || {
            called.set(true);
            async { Ok(prices(true)) }
        }),
    )
    .await;
    assert!(result.is_err());
    assert!(!called.get());
    struct Dropped<'a>(&'a Cell<bool>);
    impl Drop for Dropped<'_> {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    let dropped = Cell::new(false);
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        collect(async { Ok(availability(true)) }, || async {
            let _guard = Dropped(&dropped);
            std::future::pending().await
        }),
    )
    .await;
    assert!(result.is_err());
    assert!(dropped.get());
}

#[tokio::test]
async fn maximum_candidate_join_keeps_order_and_bounded_references() {
    let mut inventory = availability(true);
    inventory.pages.clear();
    inventory.page_limit = 100;
    inventory.node_limit = 10000;
    for page in 0..100u16 {
        inventory.pages.push(AvailablePage {
            start: None,
            next: None,
            registry_version: 2,
            observed_at_unix_seconds: 12,
            nodes: (0..100u16)
                .map(|offset| {
                    let mut node = node(Some("dc"));
                    node.node_id = Principal::from_slice(&(page * 100 + offset).to_le_bytes());
                    node
                })
                .collect(),
        });
    }
    let mut declarations = prices(true);
    declarations.prices.entries.extend([
        price(NodeType::Type4, Some(id(3)), None),
        price(NodeType::Type4, None, Some("dc")),
        price(NodeType::Type4, Some(id(3)), Some("dc")),
    ]);
    declarations
        .prices
        .entries
        .extend((0..996).map(|i| price(NodeType::Type4_1, None, Some(&i.to_string()))));
    let report = collect(async { Ok(inventory) }, || async { Ok(declarations) })
        .await
        .unwrap();
    let CandidatePricing::Read { matches, .. } = report.pricing else {
        panic!()
    };
    assert_eq!(matches.len(), 10000);
    assert!(matches.iter().all(|row| row.price_indices == [0, 1, 2, 3]));
    assert_eq!(
        matches[9999].node_id,
        Principal::from_slice(&9999u16.to_le_bytes())
    );
}
