//! Shared collection provenance and report classifications.

use serde::{Deserialize, Serialize};
use std::fmt;
#[cfg(any(feature = "nns-host", feature = "sns-host"))]
use std::path::Path;

///
/// CollectionSourceRequest
///
/// Explicit network, endpoint and collection provenance shared by canister report sources.
/// Source families retain their own network admission and transport policy.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionSourceRequest {
    /// Requested IC network identity.
    pub network: String,
    /// Explicit replica endpoint used for collection.
    pub endpoint: String,
    /// UTC collection timestamp recorded in the report.
    pub fetched_at: String,
    /// Collector identity recorded in the report.
    pub fetched_by: String,
}

impl CollectionSourceRequest {
    /// Construct source settings with explicit network and provenance.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }

    /// Construct source settings from a caller-supplied Unix collection timestamp.
    #[must_use]
    pub fn from_unix_secs(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at_unix_secs: u64,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self::new(
            network,
            endpoint,
            ic_host_text::timestamp::format_utc_timestamp_secs(fetched_at_unix_secs),
            fetched_by,
        )
    }
}

/// Compare the endpoint and collection provenance echoed by a report source.
#[cfg(any(feature = "cloud-engine-host", feature = "dashboard-host"))]
pub(crate) fn source_provenance_mismatch(
    expected: (&str, &str, &str),
    actual: (&str, &str, &str),
) -> Option<String> {
    [
        ("source_endpoint", expected.0, actual.0),
        ("fetched_at", expected.1, actual.1),
        ("fetched_by", expected.2, actual.2),
    ]
    .into_iter()
    .find_map(|(field, expected, actual)| {
        (actual != expected)
            .then(|| format!("{field} is {actual:?}, expected requested value {expected:?}"))
    })
}

///
/// SourceFuture
///
/// Caller-runtime future returned by an object-safe report source capability.
///

pub type SourceFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

///
/// ReportProvenance
///
/// Source and complete-cache metadata shared by report families.
///

pub(crate) struct ReportProvenance {
    pub(crate) data_source: ReportDataSource,
    pub(crate) cache_path: Option<String>,
    pub(crate) cache_complete: Option<bool>,
}

impl ReportProvenance {
    /// Build provenance for live reports.
    pub(crate) const fn live() -> Self {
        Self {
            data_source: ReportDataSource::Live,
            cache_path: None,
            cache_complete: None,
        }
    }

    /// Build provenance for complete-cache reports.
    #[cfg(any(feature = "nns-host", feature = "sns-host"))]
    pub(crate) fn cache(cache_path: &Path) -> Self {
        Self {
            data_source: ReportDataSource::Cache,
            cache_path: Some(cache_path.display().to_string()),
            cache_complete: Some(true),
        }
    }
}

///
/// ReportDataSource
///
/// Origin of the data exposed by a report.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportDataSource {
    /// Rows were collected from live IC query calls.
    Live,
    /// Rows were read from a complete local snapshot.
    Cache,
}

impl ReportDataSource {
    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Cache => "cache",
        }
    }
}

impl fmt::Display for ReportDataSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

///
/// ReportResultScope
///
/// Completeness boundary represented by a report view.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReportResultScope {
    /// A bounded page or detail view collected live.
    BoundedLive,
    /// A view derived from an API-exhausted complete cache.
    CompleteCache,
}

impl ReportResultScope {
    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BoundedLive => "bounded-live",
            Self::CompleteCache => "complete-cache",
        }
    }
}

impl fmt::Display for ReportResultScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
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
}
