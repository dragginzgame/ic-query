//! Module: snapshot_cache::model
//!
//! Responsibility: shared snapshot envelope and completeness DTOs.
//! Does not own: cache-file IO, path construction, or command-specific metadata.
//! Boundary: defines generic JSON shapes reused by NNS and SNS snapshot caches.

use crate::{cache::CacheCollectionCompleteness, cache_file::JsonCacheReport};
use serde::{Deserialize as SerdeDeserialize, Serialize};

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
    pub domain: String,
    pub entity: String,
    pub collection: String,
    pub scope: String,
    #[serde(flatten)]
    pub metadata: Metadata,
    pub completeness: CacheCollectionCompleteness,
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
    fn completeness(&self) -> &CacheCollectionCompleteness;

    fn snapshot_domain(&self) -> &str;

    fn snapshot_entity(&self) -> &str;

    fn snapshot_collection(&self) -> &str;

    fn snapshot_scope(&self) -> &str;
}

impl<Metadata, Data> SnapshotReport for SnapshotEnvelope<Metadata, Data> {
    fn completeness(&self) -> &CacheCollectionCompleteness {
        &self.completeness
    }

    fn snapshot_domain(&self) -> &str {
        &self.domain
    }

    fn snapshot_entity(&self) -> &str {
        &self.entity
    }

    fn snapshot_collection(&self) -> &str {
        &self.collection
    }

    fn snapshot_scope(&self) -> &str {
        &self.scope
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
#[cfg(feature = "sns-host")]
pub struct SnapshotHeader<Metadata> {
    pub schema_version: u32,
    pub network: String,
    pub domain: String,
    pub entity: String,
    pub collection: String,
    pub scope: String,
    #[serde(flatten)]
    pub metadata: Metadata,
}

#[cfg(feature = "sns-host")]
impl<Metadata> JsonCacheReport for SnapshotHeader<Metadata> {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}
