use super::*;

#[test]
fn capability_status_labels_are_stable() {
    for (status, label) in [
        (IcrcCapabilityStatus::Available, "available"),
        (IcrcCapabilityStatus::Unsupported, "unsupported"),
        (IcrcCapabilityStatus::Error, "error"),
    ] {
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            format!("\"{label}\"")
        );
        assert_eq!(status.as_str(), label);
    }
}
