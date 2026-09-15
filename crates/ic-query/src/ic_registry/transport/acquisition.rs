use super::key_family::RegistryKeyFamilyCheckpoint;
use std::{collections::BTreeMap, sync::Mutex};

///
/// SubnetCatalogProgress
///
/// Transient acquisition event; never part of snapshot authority or persisted evidence.
/// Callbacks execute synchronously on the caller's runtime and should return promptly.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogProgress {
    pub endpoint: String,
    pub query_call_count: u64,
    pub phase: SubnetCatalogProgressPhase,
}

///
/// SubnetCatalogProgressPhase
///
/// Endpoint-local progress for pinned collection, validated history pages, and value reads.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubnetCatalogProgressPhase {
    EndpointStarted,
    Pinned {
        registry_version: u64,
    },
    History {
        registry_version: u64,
        through_version: u64,
        reused: bool,
    },
    Record {
        registry_version: u64,
        key: String,
        completed: bool,
    },
    Retry {
        method: &'static str,
        next_attempt: u8,
        delay_millis: u64,
    },
}

///
/// RegistryAcquisition
///
/// Caller-owned progress sink and bounded endpoint-local history checkpoints.
///

#[derive(Default)]
pub struct RegistryAcquisition {
    pub(super) history: Mutex<BTreeMap<(String, String), RegistryKeyFamilyCheckpoint>>,
    pub(crate) progress: Option<Box<dyn Fn(SubnetCatalogProgress) + Send + Sync>>,
}
