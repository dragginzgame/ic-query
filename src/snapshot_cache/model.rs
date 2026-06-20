use crate::cache_file::JsonCacheReport;
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub const SNAPSHOT_STATUS_API_EXHAUSTED: &str = "api_exhausted";

///
/// SnapshotCompleteness
///
/// Completion metadata for a published complete snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnapshotCompleteness {
    pub status: String,
    pub page_size: u32,
    pub page_count: u32,
    pub row_count: usize,
    pub point_in_time_guaranteed: bool,
}

impl SnapshotCompleteness {
    pub fn api_exhausted(
        page_size: u32,
        page_count: u32,
        row_count: usize,
        point_in_time_guaranteed: bool,
    ) -> Self {
        Self {
            status: SNAPSHOT_STATUS_API_EXHAUSTED.to_string(),
            page_size,
            page_count,
            row_count,
            point_in_time_guaranteed,
        }
    }

    pub fn is_api_exhausted(&self) -> bool {
        self.status == SNAPSHOT_STATUS_API_EXHAUSTED
    }
}

///
/// SnapshotEnvelope
///
/// Shared JSON cache envelope for complete snapshot reports.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnapshotEnvelope<Metadata, Data> {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
    #[serde(flatten)]
    pub metadata: Metadata,
    pub completeness: SnapshotCompleteness,
    #[serde(flatten)]
    pub data: Data,
}

impl<Metadata, Data> JsonCacheReport for SnapshotEnvelope<Metadata, Data> {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

///
/// SnapshotReport
///
/// JSON cache report that exposes complete snapshot metadata.
///

pub trait SnapshotReport: JsonCacheReport {
    fn completeness(&self) -> &SnapshotCompleteness;
}

impl<Metadata, Data> SnapshotReport for SnapshotEnvelope<Metadata, Data> {
    fn completeness(&self) -> &SnapshotCompleteness {
        &self.completeness
    }
}

///
/// SnapshotHeader
///
/// Minimal snapshot metadata loaded when only header validation is needed.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize)]
pub struct SnapshotHeader<Metadata> {
    pub schema_version: u32,
    pub network: String,
    #[serde(flatten)]
    pub metadata: Metadata,
}

impl<Metadata> JsonCacheReport for SnapshotHeader<Metadata> {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}
