use super::*;

#[test]
fn canister_url_appends_path_to_endpoints_with_or_without_trailing_slash() {
    for endpoint in [
        "https://ic-api.internetcomputer.org/api/v3",
        "https://ic-api.internetcomputer.org/api/v3/",
    ] {
        assert_eq!(
            canister_url(endpoint, "ryjl3-tyaaa-aaaaa-aaaba-cai")
                .expect("canister URL")
                .as_str(),
            "https://ic-api.internetcomputer.org/api/v3/canisters/ryjl3-tyaaa-aaaaa-aaaba-cai"
        );
    }
}

#[test]
fn canister_url_rejects_query_and_fragment_components() {
    for endpoint in [
        "https://ic-api.internetcomputer.org/api/v3?limit=1",
        "https://ic-api.internetcomputer.org/api/v3#canisters",
    ] {
        assert!(matches!(
            canister_url(endpoint, "ryjl3-tyaaa-aaaaa-aaaba-cai"),
            Err(IcHostError::InvalidEndpoint { .. })
        ));
    }
}

#[test]
fn collection_urls_preserve_official_filters_and_one_bounded_page() {
    let filters = IcCanisterFilters {
        has_name: Some(true),
        subnet_id: Some(
            "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe".to_string(),
        ),
        controller_id: Some("r7inp-6aaaa-aaaaa-aaabq-cai".to_string()),
        languages: vec!["motoko".to_string(), "rust".to_string()],
        canister_types: vec!["ledger".to_string()],
        query: Some("ICP Ledger".to_string()),
    };
    let endpoint = "https://ic-api.internetcomputer.org/api/v4";

    let count =
        canister_collection_url(endpoint, &filters, CollectionOperation::Count).expect("count URL");
    let page = canister_collection_url(
        endpoint,
        &filters,
        CollectionOperation::Page {
            limit: 25,
            after: Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
            before: None,
        },
    )
    .expect("page URL");

    assert_eq!(count.path(), "/api/v4/canisters/count");
    assert_eq!(
        count
            .query_pairs()
            .filter(|(key, _)| key == "language")
            .map(|(_, value)| value.into_owned())
            .collect::<Vec<_>>(),
        ["motoko", "rust"]
    );
    assert_eq!(page.path(), "/api/v4/canisters");
    assert!(
        page.query_pairs()
            .any(|(key, value)| { key == "sort_by" && value == "canister_id" })
    );
    assert!(
        page.query_pairs()
            .any(|(key, value)| key == "limit" && value == "25")
    );
    assert!(
        page.query_pairs()
            .any(|(key, value)| { key == "after" && value == "ryjl3-tyaaa-aaaaa-aaaba-cai" })
    );
    assert!(!page.query_pairs().any(|(key, _)| key == "before"));
}

#[test]
fn collection_url_rejects_query_and_fragment_components() {
    for endpoint in [
        "https://ic-api.internetcomputer.org/api/v4?limit=1",
        "https://ic-api.internetcomputer.org/api/v4#canisters",
    ] {
        assert!(matches!(
            canister_collection_url(
                endpoint,
                &IcCanisterFilters::default(),
                CollectionOperation::Count,
            ),
            Err(IcHostError::InvalidEndpoint { .. })
        ));
    }
}

#[test]
fn wire_decoder_preserves_null_upgrade_history_and_ignores_additive_fields() {
    let wire: DashboardCanister = serde_json::from_str(
        r#"{
                "canister_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
                "canister_type": null,
                "controllers": [],
                "id": 1,
                "language": "",
                "module_hash": "",
                "name": "",
                "subnet_id": "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe",
                "updated_at": "2026-07-30T17:47:41.745647",
                "upgrades": null,
                "future_field": true
            }"#,
    )
    .expect("current Dashboard payload");

    assert_eq!(wire.canister_type, None);
    assert!(wire.upgrades.is_none());
}

#[test]
fn page_wire_decoder_preserves_controller_tuple_metadata() {
    let wire: DashboardCanisterPage = serde_json::from_str(
        r#"{
                "data": [{
                    "canister_id": "2223e-iaaaa-aaaac-awyra-cai",
                    "canister_type": null,
                    "controllers": [
                        ["lyt4m-myaaa-aaaac-aadkq-cai", ""],
                        ["r7inp-6aaaa-aaaaa-aaabq-cai", null]
                    ],
                    "id": 918419,
                    "language": "",
                    "module_hash": "",
                    "name": "",
                    "subnet_id": "4zbus-z2bmt-ilreg-xakz4-6tyre-hsqj4-slb4g-zjwqo-snjcc-iqphi-3qe",
                    "updated_at": "2026-07-31T05:13:38.882316",
                    "upgrades": [],
                    "future_field": true
                }],
                "next_cursor": "2223e-iaaaa-aaaac-awyra-cai",
                "previous_cursor": null
            }"#,
    )
    .expect("current Dashboard page payload");
    let source = wire.into_source_data(
        &IcSourceRequest::new("https://example.com/api/v4", "now", "test"),
        &IcCanisterFilters::default(),
        1,
        None,
        None,
    );

    assert_eq!(
        source.rows[0].controllers[0].raw_metadata.as_deref(),
        Some("")
    );
    assert_eq!(source.rows[0].controllers[1].raw_metadata, None);
}
