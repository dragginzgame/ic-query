use super::NnsGovernanceCollectionStatus;

#[test]
fn collection_status_json_and_display_use_the_supported_labels() {
    for (status, label) in [
        (NnsGovernanceCollectionStatus::Ready, "ready"),
        (NnsGovernanceCollectionStatus::Collecting, "collecting"),
        (NnsGovernanceCollectionStatus::Complete, "complete"),
        (
            NnsGovernanceCollectionStatus::PageLimitReached,
            "page_limit_reached",
        ),
    ] {
        let value = serde_json::json!(label);
        assert_eq!(
            serde_json::to_value(status).expect("serialize status"),
            value
        );
        assert_eq!(
            serde_json::from_value::<NnsGovernanceCollectionStatus>(value)
                .expect("deserialize status"),
            status
        );
        assert_eq!(status.as_str(), label);
        assert_eq!(status.to_string(), label);
    }
}
