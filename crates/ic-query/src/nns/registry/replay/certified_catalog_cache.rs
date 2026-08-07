//! Module: nns::registry::replay::certified_catalog_cache
//!
//! Responsibility: publish and load archive-bound catalogs under explicit local cache policy.
//! Does not own: archive collection, archive refresh, network read-through, defaults, or CLI.
//! Boundary: serialized cache content gains authority only by exact comparison with a fresh
//! projection from the caller-supplied authenticated archive.

use super::{
    NnsAuthenticatedRegistryArchive, NnsCertifiedSubnetCatalogAuthority,
    NnsCertifiedSubnetCatalogProjectionRequest, NnsRegistrySubnetCatalogProjectionError,
    project_nns_certified_subnet_catalog,
};
use crate::{
    cache_file::{
        CacheFileError, RefreshLockRequest, create_managed_parent_directory, open_managed_file,
        with_refresh_lock, write_managed_file_atomically,
    },
    hex::hex_bytes,
    subnet_catalog::{CatalogAssurance, MAINNET_NETWORK, RawSubnetCatalog},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error as ThisError;

const CERTIFIED_CATALOG_FILE_NAME: &str = "catalog.json";
const CERTIFIED_CATALOG_LOCK_FILE_NAME: &str = "refresh.lock";

/// Version of the archive-bound certified Subnet Catalog cache envelope.
pub const NNS_CERTIFIED_SUBNET_CATALOG_CACHE_SCHEMA_VERSION: u32 = 1;

///
/// NnsCertifiedSubnetCatalogCacheLocation
///
/// Caller-selected confined location and byte ceiling for one certified catalog cache.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsCertifiedSubnetCatalogCacheLocation {
    /// Capability root that confines the cache directory and files.
    pub cache_root: PathBuf,
    /// Dedicated certified-catalog directory beneath `cache_root`.
    pub cache_directory: PathBuf,
    /// Maximum canonical bytes accepted for the complete cache envelope.
    pub maximum_cache_bytes: u64,
}

impl NnsCertifiedSubnetCatalogCacheLocation {
    /// Create one explicit certified cache location without choosing a default path or ceiling.
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        cache_directory: impl Into<PathBuf>,
        maximum_cache_bytes: u64,
    ) -> Self {
        Self {
            cache_root: cache_root.into(),
            cache_directory: cache_directory.into(),
            maximum_cache_bytes,
        }
    }
}

///
/// NnsCertifiedSubnetCatalogCachePublicationRequest
///
/// Explicit location and dedicated-lock policy for one atomic cache publication.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsCertifiedSubnetCatalogCachePublicationRequest {
    /// Confined cache location and canonical byte ceiling.
    pub location: NnsCertifiedSubnetCatalogCacheLocation,
    /// Age after which an abandoned publication lock is reported as stale.
    pub lock_stale_after_seconds: u64,
}

impl NnsCertifiedSubnetCatalogCachePublicationRequest {
    /// Create one explicit publication request without selecting a lock policy by default.
    #[must_use]
    pub const fn new(
        location: NnsCertifiedSubnetCatalogCacheLocation,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            location,
            lock_stale_after_seconds,
        }
    }
}

///
/// NnsCertifiedSubnetCatalogCacheEnvelope
///
/// Serializable catalog and archive commitments; this DTO is untrusted outside a cache authority.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsCertifiedSubnetCatalogCacheEnvelope {
    /// Certified-catalog cache schema version.
    pub schema_version: u32,
    /// SHA-256 of the canonical authenticated archive manifest.
    pub archive_manifest_sha256: String,
    /// Complete canonical catalog, including Registry, assurance, certificate, and policy identity.
    /// This serde-facing field cannot self-promote its `Certified` assurance.
    pub catalog: RawSubnetCatalog,
}

#[derive(Serialize)]
struct CertifiedCatalogCacheEnvelopeRef<'a> {
    schema_version: u32,
    archive_manifest_sha256: String,
    catalog: &'a RawSubnetCatalog,
}

///
/// NnsCertifiedSubnetCatalogCacheDisposition
///
/// Observable local cache action that supplied one certified catalog authority.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsCertifiedSubnetCatalogCacheDisposition {
    /// Existing cache content exactly matched the authenticated archive projection.
    CacheHit,
    /// Missing cache content was explicitly published from the supplied archive.
    PublishedMissing,
    /// Recoverably invalid cache content was explicitly replaced from the supplied archive.
    PublishedInvalid,
    /// The caller explicitly requested unconditional atomic publication.
    ForcedPublication,
}

impl NnsCertifiedSubnetCatalogCacheDisposition {
    /// Return the stable JSON and diagnostic label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CacheHit => "cache_hit",
            Self::PublishedMissing => "published_missing",
            Self::PublishedInvalid => "published_invalid",
            Self::ForcedPublication => "forced_publication",
        }
    }
}

///
/// NnsCertifiedSubnetCatalogCacheEvidence
///
/// Compact persistable catalog, archive, certificate, and cache-action identity.
/// This serde DTO describes a successful authority but cannot reconstruct or self-assert it.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsCertifiedSubnetCatalogCacheEvidence {
    /// Exact Registry version represented by the certified catalog.
    pub registry_version: u64,
    /// Lowercase SHA-256 digest of the canonical catalog authority payload.
    pub catalog_digest: String,
    /// Assurance established by the attached authenticated archive.
    pub assurance: CatalogAssurance,
    /// Canonically ordered source endpoints retained by the archive.
    pub source_endpoints: Vec<String>,
    /// Lowercase SHA-256 of the complete canonical archive manifest.
    pub archive_manifest_sha256: String,
    /// Lowercase digest of the trusted mainnet root key used by the archive.
    pub root_key_digest: String,
    /// Lowercase commitment to the ordered authenticated report sequence.
    pub evidence_chain_digest: String,
    /// Lowercase commitment to the exact reconstructed Registry state.
    pub complete_state_digest: String,
    /// Earliest authenticated certificate time across retained archive batches.
    pub minimum_certificate_time_nanos: u64,
    /// Latest authenticated certificate time across retained archive batches.
    pub maximum_certificate_time_nanos: u64,
    /// Observable local cache action that supplied the authority.
    pub cache_disposition: NnsCertifiedSubnetCatalogCacheDisposition,
}

///
/// NnsCertifiedSubnetCatalogCacheAuthority
///
/// Cache envelope exactly matched to a fresh projection from its attached authenticated archive.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsCertifiedSubnetCatalogCacheAuthority<'a> {
    authority: NnsCertifiedSubnetCatalogAuthority<'a>,
    path: PathBuf,
    archive_manifest_sha256: String,
    disposition: NnsCertifiedSubnetCatalogCacheDisposition,
}

impl<'a> NnsCertifiedSubnetCatalogCacheAuthority<'a> {
    /// Return the freshly projected catalog authority qualifying the cached snapshot.
    #[must_use]
    pub const fn authority(&self) -> &NnsCertifiedSubnetCatalogAuthority<'a> {
        &self.authority
    }

    /// Return the exact confined cache path supplying or receiving the snapshot.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Return the observable local cache action used by this operation.
    #[must_use]
    pub const fn disposition(&self) -> NnsCertifiedSubnetCatalogCacheDisposition {
        self.disposition
    }

    /// Return compact authority evidence suitable for embedding in a durable plan.
    #[must_use]
    pub fn authority_evidence(&self) -> NnsCertifiedSubnetCatalogCacheEvidence {
        let provenance = self.authority.catalog().provenance();
        let manifest = self.authority.archive().manifest();
        NnsCertifiedSubnetCatalogCacheEvidence {
            registry_version: provenance.registry_version,
            catalog_digest: self.authority.catalog().raw().catalog_digest.clone(),
            assurance: provenance.assurance,
            source_endpoints: provenance.source_endpoints.clone(),
            archive_manifest_sha256: self.archive_manifest_sha256.clone(),
            root_key_digest: manifest.root_key_digest.clone(),
            evidence_chain_digest: manifest.evidence_chain_digest.clone(),
            complete_state_digest: manifest.complete_state_digest.clone(),
            minimum_certificate_time_nanos: manifest.minimum_certificate_time_nanos,
            maximum_certificate_time_nanos: manifest.maximum_certificate_time_nanos,
            cache_disposition: self.disposition,
        }
    }
}

///
/// NnsCertifiedSubnetCatalogCacheError
///
/// Typed projection, confinement, limit, encoding, and archive-binding failures.
///

#[derive(Debug, ThisError)]
pub enum NnsCertifiedSubnetCatalogCacheError {
    /// The authenticated archive did not qualify for certified catalog projection.
    #[error(transparent)]
    Projection(#[from] NnsRegistrySubnetCatalogProjectionError),

    /// A confined cache or publication-lock operation failed.
    #[error("certified Subnet Catalog cache filesystem operation failed: {source}")]
    FileOperation {
        /// Underlying confined cache-file failure.
        #[source]
        source: CacheFileError,
    },

    /// The caller-selected cache file does not exist.
    #[error("certified Subnet Catalog cache is missing at {path}")]
    MissingCache {
        /// Exact managed cache path that was absent.
        path: PathBuf,
    },

    /// The cache exceeded the caller-selected canonical byte ceiling.
    #[error(
        "certified Subnet Catalog cache at {path} exceeds its byte limit: actual={actual}, maximum={maximum}"
    )]
    CacheLimitExceeded {
        /// Exact managed cache path.
        path: PathBuf,
        /// Observed or encoded cache bytes.
        actual: u64,
        /// Caller-selected maximum bytes.
        maximum: u64,
    },

    /// Cache bytes are not a valid strict envelope.
    #[error("certified Subnet Catalog cache at {path} is invalid JSON: {source}")]
    InvalidJson {
        /// Exact managed cache path.
        path: PathBuf,
        /// Strict JSON decoding failure.
        #[source]
        source: serde_json::Error,
    },

    /// Cache JSON is valid but is not the one canonical compact encoding.
    #[error("certified Subnet Catalog cache at {path} is not canonical compact JSON")]
    NonCanonicalEncoding {
        /// Exact managed cache path.
        path: PathBuf,
    },

    /// The decoded cache uses an unsupported envelope schema.
    #[error(
        "unsupported certified Subnet Catalog cache schema {found}; supported schema is {supported}"
    )]
    UnsupportedSchemaVersion {
        /// Schema found in the cache.
        found: u32,
        /// Schema implemented by this library version.
        supported: u32,
    },

    /// Cache content does not exactly match the caller-supplied authenticated archive projection.
    #[error(
        "certified Subnet Catalog cache field {field} does not match the supplied authenticated Registry archive projection"
    )]
    ArchiveBindingMismatch {
        /// First deterministic envelope field that disagreed.
        field: &'static str,
    },

    /// Canonical serialization or byte accounting failed.
    #[error("certified Subnet Catalog cache serialization failed: {source}")]
    Serialization {
        /// JSON serialization failure.
        #[source]
        source: serde_json::Error,
    },

    /// A platform byte count cannot be represented by the public accounting type.
    #[error("certified Subnet Catalog cache byte accounting overflowed")]
    Accounting,
}

/// Return the fixed cache-envelope path beneath a caller-selected dedicated directory.
#[must_use]
pub fn nns_certified_subnet_catalog_cache_path(cache_directory: &Path) -> PathBuf {
    cache_directory.join(CERTIFIED_CATALOG_FILE_NAME)
}

/// Return the dedicated publication-lock path beneath a caller-selected cache directory.
#[must_use]
pub fn nns_certified_subnet_catalog_cache_refresh_lock_path(cache_directory: &Path) -> PathBuf {
    cache_directory.join(CERTIFIED_CATALOG_LOCK_FILE_NAME)
}

/// Atomically publish a cache derived only from one qualifying authenticated archive projection.
///
/// Projection completes before any filesystem mutation. The operation makes no network call and
/// preserves any prior cache if qualification, lock acquisition, serialization, or replacement
/// fails. A lock-release error can be reported after the atomic replacement has completed.
pub fn publish_nns_certified_subnet_catalog_cache<'a>(
    archive: &'a NnsAuthenticatedRegistryArchive,
    projection_request: &NnsCertifiedSubnetCatalogProjectionRequest,
    request: &NnsCertifiedSubnetCatalogCachePublicationRequest,
) -> Result<NnsCertifiedSubnetCatalogCacheAuthority<'a>, NnsCertifiedSubnetCatalogCacheError> {
    publish_cache_with_disposition(
        archive,
        projection_request,
        request,
        NnsCertifiedSubnetCatalogCacheDisposition::ForcedPublication,
    )
}

fn publish_cache_with_disposition<'a>(
    archive: &'a NnsAuthenticatedRegistryArchive,
    projection_request: &NnsCertifiedSubnetCatalogProjectionRequest,
    request: &NnsCertifiedSubnetCatalogCachePublicationRequest,
    disposition: NnsCertifiedSubnetCatalogCacheDisposition,
) -> Result<NnsCertifiedSubnetCatalogCacheAuthority<'a>, NnsCertifiedSubnetCatalogCacheError> {
    let authority = project_nns_certified_subnet_catalog(archive, projection_request)?;
    let envelope = cache_envelope(&authority)?;
    let cache_path = nns_certified_subnet_catalog_cache_path(&request.location.cache_directory);
    let lock_path =
        nns_certified_subnet_catalog_cache_refresh_lock_path(&request.location.cache_directory);
    let encoded_length = canonical_serialized_len(&envelope)?;
    enforce_cache_limit(
        &cache_path,
        encoded_length,
        request.location.maximum_cache_bytes,
    )?;
    create_managed_parent_directory(&request.location.cache_root, &cache_path)
        .map_err(file_operation)?;
    with_refresh_lock(
        RefreshLockRequest {
            cache_root: &request.location.cache_root,
            lock_path: &lock_path,
            target_path: &cache_path,
            network: MAINNET_NETWORK,
            now_unix_secs: projection_request.validation.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        file_operation,
        || {
            write_managed_file_atomically(&request.location.cache_root, &cache_path, |file| {
                serde_json::to_writer(file, &envelope).map_err(json_io_error)
            })
            .map_err(file_operation)
        },
    )?;
    let archive_manifest_sha256 = envelope.archive_manifest_sha256;
    Ok(NnsCertifiedSubnetCatalogCacheAuthority {
        authority,
        path: cache_path,
        archive_manifest_sha256,
        disposition,
    })
}

/// Load a bounded cache and match it to a fresh projection from an authenticated archive.
///
/// This operation is cache-only and local-only. It never refreshes, repairs, publishes, or makes
/// a source call. Serialized `Certified` provenance is not admitted unless every envelope and
/// catalog field equals the projection recomputed from `archive` under `projection_request`.
pub fn load_nns_certified_subnet_catalog_cache<'a>(
    archive: &'a NnsAuthenticatedRegistryArchive,
    projection_request: &NnsCertifiedSubnetCatalogProjectionRequest,
    location: &NnsCertifiedSubnetCatalogCacheLocation,
) -> Result<NnsCertifiedSubnetCatalogCacheAuthority<'a>, NnsCertifiedSubnetCatalogCacheError> {
    let cache_path = nns_certified_subnet_catalog_cache_path(&location.cache_directory);
    let bytes = read_bounded_cache(location, &cache_path)?.ok_or_else(|| {
        NnsCertifiedSubnetCatalogCacheError::MissingCache {
            path: cache_path.clone(),
        }
    })?;
    let envelope: NnsCertifiedSubnetCatalogCacheEnvelope =
        serde_json::from_slice(&bytes).map_err(|source| {
            NnsCertifiedSubnetCatalogCacheError::InvalidJson {
                path: cache_path.clone(),
                source,
            }
        })?;
    if envelope.schema_version != NNS_CERTIFIED_SUBNET_CATALOG_CACHE_SCHEMA_VERSION {
        return Err(
            NnsCertifiedSubnetCatalogCacheError::UnsupportedSchemaVersion {
                found: envelope.schema_version,
                supported: NNS_CERTIFIED_SUBNET_CATALOG_CACHE_SCHEMA_VERSION,
            },
        );
    }
    if !is_canonical_encoding(&envelope, &bytes)? {
        return Err(NnsCertifiedSubnetCatalogCacheError::NonCanonicalEncoding { path: cache_path });
    }
    let authority = project_nns_certified_subnet_catalog(archive, projection_request)?;
    let expected = cache_envelope(&authority)?;
    if let Some(field) = first_envelope_mismatch(&envelope, &expected) {
        return Err(NnsCertifiedSubnetCatalogCacheError::ArchiveBindingMismatch { field });
    }
    Ok(NnsCertifiedSubnetCatalogCacheAuthority {
        authority,
        path: cache_path,
        archive_manifest_sha256: envelope.archive_manifest_sha256,
        disposition: NnsCertifiedSubnetCatalogCacheDisposition::CacheHit,
    })
}

/// Load existing cache content or publish only when the cache is missing.
///
/// This local-only operation never replaces invalid content and never refreshes the supplied
/// archive. Its name makes the only authorized cache mutation explicit.
pub fn load_or_publish_missing_nns_certified_subnet_catalog_cache<'a>(
    archive: &'a NnsAuthenticatedRegistryArchive,
    projection_request: &NnsCertifiedSubnetCatalogProjectionRequest,
    request: &NnsCertifiedSubnetCatalogCachePublicationRequest,
) -> Result<NnsCertifiedSubnetCatalogCacheAuthority<'a>, NnsCertifiedSubnetCatalogCacheError> {
    match load_nns_certified_subnet_catalog_cache(archive, projection_request, &request.location) {
        Err(NnsCertifiedSubnetCatalogCacheError::MissingCache { .. }) => {
            publish_cache_with_disposition(
                archive,
                projection_request,
                request,
                NnsCertifiedSubnetCatalogCacheDisposition::PublishedMissing,
            )
        }
        result => result,
    }
}

/// Load existing cache content or publish when it is missing or recoverably invalid.
///
/// Recoverable invalidity is limited to bounded content, JSON, schema, canonical-encoding, and
/// archive-binding failures. Filesystem, projection, serialization, and accounting errors are
/// returned unchanged. Republication cannot make stale archive evidence fresh.
pub fn load_or_publish_missing_or_invalid_nns_certified_subnet_catalog_cache<'a>(
    archive: &'a NnsAuthenticatedRegistryArchive,
    projection_request: &NnsCertifiedSubnetCatalogProjectionRequest,
    request: &NnsCertifiedSubnetCatalogCachePublicationRequest,
) -> Result<NnsCertifiedSubnetCatalogCacheAuthority<'a>, NnsCertifiedSubnetCatalogCacheError> {
    match load_nns_certified_subnet_catalog_cache(archive, projection_request, &request.location) {
        Err(error) => {
            let disposition = match error {
                NnsCertifiedSubnetCatalogCacheError::MissingCache { .. } => {
                    NnsCertifiedSubnetCatalogCacheDisposition::PublishedMissing
                }
                NnsCertifiedSubnetCatalogCacheError::CacheLimitExceeded { .. }
                | NnsCertifiedSubnetCatalogCacheError::InvalidJson { .. }
                | NnsCertifiedSubnetCatalogCacheError::NonCanonicalEncoding { .. }
                | NnsCertifiedSubnetCatalogCacheError::UnsupportedSchemaVersion { .. }
                | NnsCertifiedSubnetCatalogCacheError::ArchiveBindingMismatch { .. } => {
                    NnsCertifiedSubnetCatalogCacheDisposition::PublishedInvalid
                }
                error => return Err(error),
            };
            publish_cache_with_disposition(archive, projection_request, request, disposition)
        }
        result => result,
    }
}

fn first_envelope_mismatch(
    cached: &NnsCertifiedSubnetCatalogCacheEnvelope,
    expected: &CertifiedCatalogCacheEnvelopeRef<'_>,
) -> Option<&'static str> {
    if cached.schema_version != expected.schema_version {
        return Some("schema_version");
    }
    if cached.archive_manifest_sha256 != expected.archive_manifest_sha256 {
        return Some("archive_manifest_sha256");
    }
    if &cached.catalog != expected.catalog {
        return Some("catalog");
    }
    None
}

fn cache_envelope<'a>(
    authority: &'a NnsCertifiedSubnetCatalogAuthority<'_>,
) -> Result<CertifiedCatalogCacheEnvelopeRef<'a>, NnsCertifiedSubnetCatalogCacheError> {
    let manifest = authority.archive().manifest();
    let catalog = authority.catalog().raw();
    Ok(CertifiedCatalogCacheEnvelopeRef {
        schema_version: NNS_CERTIFIED_SUBNET_CATALOG_CACHE_SCHEMA_VERSION,
        archive_manifest_sha256: canonical_sha256(manifest)?,
        catalog,
    })
}

fn canonical_sha256(value: &impl Serialize) -> Result<String, NnsCertifiedSubnetCatalogCacheError> {
    let mut digest = Sha256::new();
    serde_json::to_writer(&mut digest, value)
        .map_err(|source| NnsCertifiedSubnetCatalogCacheError::Serialization { source })?;
    Ok(hex_bytes(&digest.finalize()))
}

fn canonical_serialized_len(
    value: &impl Serialize,
) -> Result<u64, NnsCertifiedSubnetCatalogCacheError> {
    let mut writer = CountingWriter::default();
    serde_json::to_writer(&mut writer, value)
        .map_err(|source| NnsCertifiedSubnetCatalogCacheError::Serialization { source })?;
    Ok(writer.bytes)
}

fn is_canonical_encoding(
    value: &impl Serialize,
    bytes: &[u8],
) -> Result<bool, NnsCertifiedSubnetCatalogCacheError> {
    let mut writer = MatchingWriter::new(bytes);
    serde_json::to_writer(&mut writer, value)
        .map_err(|source| NnsCertifiedSubnetCatalogCacheError::Serialization { source })?;
    Ok(writer.is_complete_match())
}

fn read_bounded_cache(
    location: &NnsCertifiedSubnetCatalogCacheLocation,
    path: &Path,
) -> Result<Option<Vec<u8>>, NnsCertifiedSubnetCatalogCacheError> {
    let Some(mut file) = open_managed_file(&location.cache_root, path).map_err(file_operation)?
    else {
        return Ok(None);
    };
    let metadata_length = file
        .metadata()
        .map_err(|source| {
            file_operation(CacheFileError::OpenManagedPath {
                root: location.cache_root.clone(),
                path: path.to_path_buf(),
                source,
            })
        })?
        .len();
    enforce_cache_limit(path, metadata_length, location.maximum_cache_bytes)?;
    let capacity = usize::try_from(metadata_length)
        .map_err(|_| NnsCertifiedSubnetCatalogCacheError::Accounting)?;
    let mut bytes = Vec::with_capacity(capacity);
    Read::by_ref(&mut file)
        .take(location.maximum_cache_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| {
            file_operation(CacheFileError::OpenManagedPath {
                root: location.cache_root.clone(),
                path: path.to_path_buf(),
                source,
            })
        })?;
    let actual =
        u64::try_from(bytes.len()).map_err(|_| NnsCertifiedSubnetCatalogCacheError::Accounting)?;
    enforce_cache_limit(path, actual, location.maximum_cache_bytes)?;
    Ok(Some(bytes))
}

fn enforce_cache_limit(
    path: &Path,
    actual: u64,
    maximum: u64,
) -> Result<(), NnsCertifiedSubnetCatalogCacheError> {
    if actual > maximum {
        return Err(NnsCertifiedSubnetCatalogCacheError::CacheLimitExceeded {
            path: path.to_path_buf(),
            actual,
            maximum,
        });
    }
    Ok(())
}

const fn file_operation(source: CacheFileError) -> NnsCertifiedSubnetCatalogCacheError {
    NnsCertifiedSubnetCatalogCacheError::FileOperation { source }
}

fn json_io_error(source: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, source)
}

#[derive(Default)]
struct CountingWriter {
    bytes: u64,
}

impl Write for CountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let length = u64::try_from(bytes.len())
            .map_err(|_| io::Error::other("serialized byte count overflowed"))?;
        self.bytes = self
            .bytes
            .checked_add(length)
            .ok_or_else(|| io::Error::other("serialized byte count overflowed"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct MatchingWriter<'a> {
    expected: &'a [u8],
    position: usize,
    matches: bool,
}

impl<'a> MatchingWriter<'a> {
    const fn new(expected: &'a [u8]) -> Self {
        Self {
            expected,
            position: 0,
            matches: true,
        }
    }

    const fn is_complete_match(&self) -> bool {
        self.matches && self.position == self.expected.len()
    }
}

impl Write for MatchingWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let end = self
            .position
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("serialized byte count overflowed"))?;
        if self.expected.get(self.position..end) != Some(bytes) {
            self.matches = false;
        }
        self.position = end;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
