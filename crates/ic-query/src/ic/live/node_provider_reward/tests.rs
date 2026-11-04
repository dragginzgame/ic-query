use super::*;

#[test]
fn reward_urls_preserve_explicit_page_and_history_bounds() {
    let list = node_provider_reward_list_url(
        "https://ic-api.internetcomputer.org/api/v3/",
        &IcNodeProviderRewardListQuery::new(25, 50, Some(6_470)),
    )
    .expect("list URL");
    assert_eq!(
        list.as_str(),
        "https://ic-api.internetcomputer.org/api/v3/reward-node-providers?format=json&limit=25&offset=50&max_reward_node_provider_index=6470"
    );

    let history = node_provider_reward_history_url(
        "https://ic-api.internetcomputer.org/api/v3",
        &IcNodeProviderRewardHistoryQuery::new(1_783_900_000, 1_784_300_000, 86_400),
    )
    .expect("history URL");
    assert_eq!(history.path(), "/api/v3/timeseries/reward-node-providers");
    assert_eq!(
        history.query(),
        Some("format=json&start=1783900000&end=1784300000&step=86400")
    );
    assert_eq!(
        node_provider_reward_info_url("https://example.com/api/v3", 7_562)
            .expect("info URL")
            .path(),
        "/api/v3/reward-node-providers/7562"
    );
}

#[test]
fn wire_preserves_current_and_historical_shapes() {
    let current: DashboardNodeProviderReward = serde_json::from_str(
            r#"{
                "amount_e8s": 1583574085000,
                "details": {"to_account":"0000000000000000000000000000000000000000000000000000000000000000"},
                "id": 7562,
                "maximum_node_provider_rewards_e8s": 10000000000000,
                "minimum_xdr_permyriad_per_icp": 20000,
                "node_provider": "rrkah-fqaaa-aaaaa-aaaaq-cai",
                "proposal_id": null,
                "registry_version": null,
                "reward_mode": "RewardToAccount",
                "timestamp_seconds": 1784081341,
                "updated_at": "2026-07-15T04:30:01.558435",
                "xdr_conversion_rate": {"timestamp_seconds":1784073600,"xdr_permyriad_per_icp":16379}
            }"#,
        )
        .expect("current wire");
    let current = current.into_public().expect("current public row");
    assert_eq!(current.reward_id, 7_562);
    assert_eq!(
        current.xdr_conversion_rate.xdr_permyriad_per_icp,
        Some(16_379)
    );

    let historical: DashboardNodeProviderReward = serde_json::from_str(
        r#"{
                "amount_e8s": 1,
                "details": {},
                "id": 707,
                "maximum_node_provider_rewards_e8s": null,
                "minimum_xdr_permyriad_per_icp": null,
                "node_provider": "rrkah-fqaaa-aaaaa-aaaaq-cai",
                "proposal_id": 10270.0,
                "registry_version": null,
                "reward_mode": "RewardToAccount",
                "timestamp_seconds": 1620000000,
                "updated_at": "2021-05-03T00:00:00",
                "xdr_conversion_rate": {}
            }"#,
    )
    .expect("historical wire");
    let historical = historical.into_public().expect("historical public row");
    assert_eq!(historical.proposal_id, Some(10_270));
    assert_eq!(historical.xdr_conversion_rate.timestamp_unix_secs, None);
}

#[test]
fn integer_decoder_rejects_fractional_values() {
    assert_eq!(
        number_to_u64("value", serde_json::from_str("10270.0").expect("number"))
            .expect("integral float"),
        10_270
    );
    assert!(number_to_u64("value", serde_json::from_str("1.5").expect("number")).is_err());
    assert!(
        number_to_u64(
            "value",
            serde_json::from_str("9007199254740993.0").expect("number")
        )
        .is_err()
    );
}
