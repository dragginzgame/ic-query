use super::*;

#[test]
fn typed_statuses_keep_stable_json_labels() {
    for (status, expected) in [
        (CacheValidationStatus::Valid, "ok"),
        (CacheValidationStatus::Invalid, "invalid"),
    ] {
        assert_eq!(status.as_str(), expected);
        assert_eq!(status.to_string(), expected);
        assert_eq!(
            serde_json::to_value(status).expect("serialize cache validation status"),
            serde_json::json!(expected)
        );
    }
    for (status, expected) in [
        (CacheRefreshAttemptStatus::Running, "running"),
        (CacheRefreshAttemptStatus::Complete, "complete"),
        (CacheRefreshAttemptStatus::Failed, "failed"),
    ] {
        assert_eq!(status.as_str(), expected);
        assert_eq!(status.to_string(), expected);
        assert_eq!(
            serde_json::from_value::<CacheRefreshAttemptStatus>(serde_json::json!(expected))
                .expect("deserialize refresh-attempt status"),
            status
        );
        assert_eq!(
            serde_json::to_value(status).expect("serialize refresh-attempt status"),
            serde_json::json!(expected)
        );
    }
    assert!(
        serde_json::from_value::<CacheRefreshAttemptStatus>(serde_json::json!("unknown")).is_err()
    );
}
