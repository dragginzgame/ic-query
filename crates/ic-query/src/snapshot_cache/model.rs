//! Module: snapshot_cache::model
//!
//! Responsibility: shared snapshot envelope and completeness DTOs.
//! Does not own: cache-file IO, path construction, or command-specific metadata.
//! Boundary: defines generic JSON shapes reused by NNS and SNS snapshot caches.

use crate::cache_file::JsonCacheReport;
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub const SNAPSHOT_STATUS_API_EXHAUSTED: &str = "api_exhausted";
pub const SNAPSHOT_CACHE_STATUS_INVALID: &str = "invalid";
pub const SNAPSHOT_CACHE_STATUS_OK: &str = "ok";

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
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

    fn snapshot_domain(&self) -> Option<&str> {
        None
    }

    fn snapshot_entity(&self) -> Option<&str> {
        None
    }

    fn snapshot_collection(&self) -> Option<&str> {
        None
    }

    fn snapshot_scope(&self) -> Option<&str> {
        None
    }
}

impl<Metadata, Data> SnapshotReport for SnapshotEnvelope<Metadata, Data> {
    fn completeness(&self) -> &SnapshotCompleteness {
        &self.completeness
    }

    fn snapshot_domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    fn snapshot_entity(&self) -> Option<&str> {
        self.entity.as_deref()
    }

    fn snapshot_collection(&self) -> Option<&str> {
        self.collection.as_deref()
    }

    fn snapshot_scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }
}

///
/// SnapshotIdentityMismatch
///
/// Mismatch between a snapshot envelope identity field and its logical key.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotIdentityMismatch {
    pub field: &'static str,
    pub expected: String,
    pub actual: String,
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
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default)]
    pub collection: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
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
