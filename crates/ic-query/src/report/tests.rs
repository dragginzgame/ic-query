use super::{ReportDataSource, ReportResultScope};

#[cfg(any(feature = "cloud-engine-host", feature = "dashboard-host"))]
#[test]
fn source_provenance_comparison_preserves_field_order() {
    let expected = ("endpoint", "time", "collector");
    assert_eq!(super::source_provenance_mismatch(expected, expected), None);
    for (actual, field) in [
        (("other", "other", "other"), "source_endpoint"),
        (("endpoint", "other", "other"), "fetched_at"),
        (("endpoint", "time", "other"), "fetched_by"),
    ] {
        let reason = super::source_provenance_mismatch(expected, actual).unwrap();
        assert!(reason.starts_with(&format!("{field} is ")));
    }
}

#[test]
fn report_origins_round_trip_and_require_a_cache_path() {
    for provenance in [
        super::ReportProvenance::Live,
        super::ReportProvenance::Cache {
            cache_path: "/cache/snapshot.json".to_string(),
        },
    ] {
        let value = serde_json::to_value(&provenance).unwrap();
        assert_eq!(
            serde_json::from_value::<super::ReportProvenance>(value).unwrap(),
            provenance
        );
    }
    assert!(
        serde_json::from_value::<super::ReportProvenance>(serde_json::json!({
            "data_source": "cache"
        }))
        .is_err()
    );
}

#[test]
fn report_provenance_labels_round_trip() {
    for (source, label) in [
        (ReportDataSource::Live, "live"),
        (ReportDataSource::Cache, "cache"),
    ] {
        assert_eq!(source.as_str(), label);
        assert_eq!(source.to_string(), label);
        assert_eq!(
            serde_json::to_string(&source).expect("serialize report data source"),
            format!("\"{label}\"")
        );
        assert_eq!(
            serde_json::from_str::<ReportDataSource>(&format!("\"{label}\""))
                .expect("deserialize report data source"),
            source
        );
    }
    for (scope, label) in [
        (ReportResultScope::BoundedLive, "bounded-live"),
        (ReportResultScope::CompleteCache, "complete-cache"),
    ] {
        assert_eq!(scope.as_str(), label);
        assert_eq!(scope.to_string(), label);
        assert_eq!(
            serde_json::to_string(&scope).expect("serialize report result scope"),
            format!("\"{label}\"")
        );
        assert_eq!(
            serde_json::from_str::<ReportResultScope>(&format!("\"{label}\""))
                .expect("deserialize report result scope"),
            scope
        );
    }
    assert!(serde_json::from_str::<ReportDataSource>("\"api\"").is_err());
    assert!(serde_json::from_str::<ReportResultScope>("\"partial\"").is_err());
}
