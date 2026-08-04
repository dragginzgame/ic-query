//! Shared report provenance classifications.

use serde::{Deserialize, Serialize};
use std::fmt;

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
