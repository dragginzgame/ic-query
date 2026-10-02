use super::history_cache::{RegistryHistoryCache, RegistryHistoryCacheDisposition};
use super::key_family::RegistryKeyFamilyCheckpoint;
use std::path::PathBuf;
use std::{collections::BTreeMap, sync::Mutex};

///
/// SubnetCatalogProgress
///
/// Transient acquisition event; never part of snapshot authority or persisted evidence.
/// Callbacks execute synchronously on the caller's runtime and should return promptly.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogProgress {
    /// Exact endpoint supplying this collection's Registry evidence.
    pub endpoint: String,
    /// Endpoint-local explicit query attempts, including catalog retries but not
    /// ic-agent's internal HTTP retries or ancillary verification requests.
    pub query_call_count: u64,
    /// Collection operation or validated history watermark being reported.
    pub phase: SubnetCatalogProgressPhase,
}

///
/// SubnetCatalogProgressPhase
///
/// Endpoint-local progress for pinned collection, validated history pages, and value reads.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubnetCatalogProgressPhase {
    /// Endpoint collection has started, before agent construction and version acquisition.
    EndpointStarted,
    /// The endpoint's latest Registry version has been selected for all record reads.
    Pinned {
        /// Exact Registry version pinned by this endpoint.
        registry_version: u64,
    },
    /// History discovery has started, resumed, or validated another complete page.
    History {
        /// Target Registry version whose key-family membership is being reconstructed.
        registry_version: u64,
        /// Highest completely validated version, capped at the target; zero at a cold start.
        through_version: u64,
        /// Whether this event reports a retained endpoint-local history prefix.
        reused: bool,
    },
    /// A caller-owned disk history checkpoint was examined or published.
    HistoryCache {
        /// Confined file containing endpoint-isolated history transcripts.
        path: PathBuf,
        /// Observable checkpoint read or write result.
        disposition: RegistryHistoryCacheDisposition,
        /// Highest validated retained version for this endpoint and prefix.
        through_version: u64,
        /// Rejection or skipped-publication reason when applicable.
        reason: Option<String>,
    },
    /// A pinned Registry value read has started or completed.
    Record {
        /// Requested Registry version, distinct from the value's last mutation version.
        registry_version: u64,
        /// Exact Registry key being acquired.
        key: String,
        /// True after value transport and decoding complete; final catalog validation
        /// may still fail after this event.
        completed: bool,
    },
    /// A transient query failure will be retried after cancellable backoff.
    Retry {
        /// Registry query method that will be invoked again with the same inputs.
        method: &'static str,
        /// One-based attempt number for the upcoming query invocation.
        next_attempt: u8,
        /// Backoff duration in milliseconds before the next attempt.
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
    pub(crate) progress: Option<std::sync::Arc<dyn Fn(SubnetCatalogProgress) + Send + Sync>>,
    pub(super) history_cache: Option<RegistryHistoryCache>,
}

impl RegistryAcquisition {
    pub(crate) fn history_cache_paths(&self) -> Option<(&std::path::Path, &std::path::Path)> {
        self.history_cache.as_ref().map(RegistryHistoryCache::paths)
    }
    pub(crate) fn with_history_cache(
        root: PathBuf,
        progress: Option<std::sync::Arc<dyn Fn(SubnetCatalogProgress) + Send + Sync>>,
    ) -> Self {
        Self {
            progress,
            history_cache: Some(RegistryHistoryCache::new(root)),
            ..Self::default()
        }
    }
}
