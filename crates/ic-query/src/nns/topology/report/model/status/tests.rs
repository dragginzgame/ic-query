use super::*;

#[test]
fn assessment_status_labels_round_trip() {
    assert_status_labels(&[
        (NnsTopologyAssessmentStatus::Ok, "ok"),
        (NnsTopologyAssessmentStatus::Attention, "attention"),
    ]);
}

#[test]
fn capacity_status_labels_round_trip() {
    assert_status_labels(&[
        (NnsTopologyCapacityStatus::Over, "over"),
        (NnsTopologyCapacityStatus::Unknown, "unknown"),
        (NnsTopologyCapacityStatus::Full, "full"),
        (NnsTopologyCapacityStatus::Available, "available"),
    ]);
}

#[test]
fn provider_status_labels_round_trip() {
    assert_status_labels(&[
        (
            NnsTopologyProviderStatus::UnknownProvider,
            "unknown_provider",
        ),
        (NnsTopologyProviderStatus::Over, "over"),
        (NnsTopologyProviderStatus::Unused, "unused"),
        (NnsTopologyProviderStatus::Ok, "ok"),
    ]);
}

fn assert_status_labels<T>(cases: &[(T, &str)])
where
    T: Copy + std::fmt::Debug + Eq + serde::Serialize + serde::de::DeserializeOwned,
{
    for &(status, label) in cases {
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            format!("\"{label}\"")
        );
        assert_eq!(
            serde_json::from_str::<T>(&format!("\"{label}\"")).unwrap(),
            status
        );
    }
}
