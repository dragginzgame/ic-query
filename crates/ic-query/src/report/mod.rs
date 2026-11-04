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

///
/// SortDirection
///
/// Direction shared by local report sorts; each sort key owns its default.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SortDirection {
    /// Order values from lowest to highest.
    Asc,
    /// Order values from highest to lowest.
    #[default]
    Desc,
}

impl SortDirection {
    /// Return the stable report label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "data_source", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReportProvenance {
    /// Rows collected directly from their source.
    Live,
    /// Rows read from one admitted complete snapshot.
    Cache {
        /// Complete snapshot path.
        cache_path: String,
    },
}

impl ReportProvenance {
    /// Build provenance for live reports.
    pub(crate) const fn live() -> Self {
        Self::Live
    }

    /// Build provenance for complete-cache reports.
    #[cfg(any(feature = "nns-host", feature = "sns-host"))]
    pub(crate) fn cache(cache_path: &Path) -> Self {
        Self::Cache {
            cache_path: cache_path.display().to_string(),
        }
    }

    /// Return the origin of these rows.
    #[must_use]
    pub const fn data_source(&self) -> ReportDataSource {
        match self {
            Self::Live => ReportDataSource::Live,
            Self::Cache { .. } => ReportDataSource::Cache,
        }
    }

    /// Return the complete snapshot path for cached rows.
    #[must_use]
    pub fn cache_path(&self) -> Option<&str> {
        match self {
            Self::Live => None,
            Self::Cache { cache_path } => Some(cache_path),
        }
    }

    /// Return API-exhaustion evidence for cached rows.
    #[must_use]
    pub const fn cache_complete(&self) -> Option<bool> {
        match self {
            Self::Live => None,
            Self::Cache { .. } => Some(true),
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
mod tests;
