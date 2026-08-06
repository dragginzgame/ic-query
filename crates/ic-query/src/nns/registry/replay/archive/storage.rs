//! Module: nns::registry::replay::archive::storage
//!
//! Responsibility: confined atomic archive publication and bounded sequential restoration.
//! Does not own: live collection, refresh policy, cache defaults, CLI, or catalog promotion.
//! Boundary: a manifest becomes usable only after every retained object is reauthenticated.

use super::{
    NnsCertifiedRegistryArchiveBatchDescriptor, NnsCertifiedRegistryArchiveError,
    NnsCertifiedRegistryArchiveLimits, NnsCertifiedRegistryArchiveManifest,
    NnsCertifiedRegistryArchiveManifestBuilder,
};
use crate::{
    cache_file::{CacheFileError, open_managed_file, write_managed_file_atomically},
    hex::hex_bytes,
    nns::registry::{
        NnsAuthenticatedRegistryDeltaBatch, NnsAuthenticatedRegistryReplaySession,
        NnsCertifiedRegistryDeltaBatchReport, NnsCertifiedRegistryDeltaBatchRequest,
        NnsRegistryHostError, NnsRegistryReplayProgress, NnsRegistryReplaySessionLimits,
        reauthenticate_nns_certified_registry_delta_batch,
    },
    subnet_catalog::parse_utc_timestamp_secs,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error as ThisError;

const ARCHIVE_MANIFEST_FILE_NAME: &str = "manifest.json";
const ARCHIVE_OBJECTS_DIRECTORY_NAME: &str = "objects";

///
/// NnsCertifiedRegistryArchiveStorageLimits
///
/// Explicit manifest and retained-object ceilings applied during publication and loading.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NnsCertifiedRegistryArchiveStorageLimits {
    /// Maximum canonical bytes accepted for the archive manifest.
    pub max_manifest_bytes: u64,
    /// Batch-count and canonical retained-report byte ceilings.
    pub archive: NnsCertifiedRegistryArchiveLimits,
}

impl NnsCertifiedRegistryArchiveStorageLimits {
    /// Create explicit storage ceilings without choosing filesystem or history defaults.
    #[must_use]
    pub const fn new(max_manifest_bytes: u64, archive: NnsCertifiedRegistryArchiveLimits) -> Self {
        Self {
            max_manifest_bytes,
            archive,
        }
    }
}

///
/// NnsAuthenticatedRegistryArchive
///
/// Structurally matched archive manifest and fully reauthenticated exact-target replay session.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsAuthenticatedRegistryArchive {
    manifest: NnsCertifiedRegistryArchiveManifest,
    replay_session: NnsAuthenticatedRegistryReplaySession,
}

impl NnsAuthenticatedRegistryArchive {
    /// Return the recomputed archive manifest matched against retained objects.
    #[must_use]
    pub const fn manifest(&self) -> &NnsCertifiedRegistryArchiveManifest {
        &self.manifest
    }

    /// Return the complete replay capability recovered by local authentication.
    #[must_use]
    pub const fn replay_session(&self) -> &NnsAuthenticatedRegistryReplaySession {
        &self.replay_session
    }

    /// Consume the archive result into its manifest and authenticated replay capability.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        NnsCertifiedRegistryArchiveManifest,
        NnsAuthenticatedRegistryReplaySession,
    ) {
        (self.manifest, self.replay_session)
    }
}

///
/// NnsCertifiedRegistryArchivePublisher
///
/// Streaming publisher that makes report objects durable before atomically replacing a manifest.
///

#[derive(Debug)]
pub struct NnsCertifiedRegistryArchivePublisher {
    cache_root: PathBuf,
    archive_root: PathBuf,
    storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
    manifest_builder: NnsCertifiedRegistryArchiveManifestBuilder,
    poisoned: bool,
}

impl NnsCertifiedRegistryArchivePublisher {
    /// Create an unpublished archive rooted beneath an explicit managed cache capability.
    #[must_use]
    pub fn new(
        cache_root: &Path,
        archive_root: &Path,
        replay_limits: NnsRegistryReplaySessionLimits,
        storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
    ) -> Self {
        Self {
            cache_root: cache_root.to_path_buf(),
            archive_root: archive_root.to_path_buf(),
            storage_limits,
            manifest_builder: NnsCertifiedRegistryArchiveManifestBuilder::new(
                replay_limits,
                storage_limits.archive,
            ),
            poisoned: false,
        }
    }

    /// Apply and durably publish one authenticated retained report object.
    ///
    /// If object publication fails after replay admission, the publisher is poisoned and must
    /// be discarded. Its manifest remains unpublished, so any prior complete archive is intact.
    pub fn apply_batch(
        &mut self,
        batch: &NnsAuthenticatedRegistryDeltaBatch<'_>,
    ) -> Result<NnsRegistryReplayProgress, NnsCertifiedRegistryArchiveStorageError> {
        self.ensure_usable()?;
        let progress = self.manifest_builder.apply_batch(batch)?;
        let Some(descriptor) = self.manifest_builder.latest_batch_descriptor().cloned() else {
            self.poisoned = true;
            return Err(NnsCertifiedRegistryArchiveError::InvalidManifest {
                reason: "accepted archive batch did not publish a descriptor".to_string(),
            }
            .into());
        };
        if let Err(error) = write_report_object(
            &self.cache_root,
            &self.archive_root,
            &descriptor,
            batch.report(),
        ) {
            self.poisoned = true;
            return Err(error);
        }
        Ok(progress)
    }

    /// Atomically publish the manifest after every authenticated report object is durable.
    pub fn finish(
        self,
    ) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveStorageError> {
        self.ensure_usable()?;
        let (manifest, replay_session) = self.manifest_builder.finish()?;
        write_manifest(
            &self.cache_root,
            &self.archive_root,
            &manifest,
            self.storage_limits.max_manifest_bytes,
        )?;
        Ok(NnsAuthenticatedRegistryArchive {
            manifest,
            replay_session,
        })
    }

    /// Return in-memory replay progress without reading or publishing the final manifest.
    #[must_use]
    pub const fn replay_session(&self) -> &super::super::NnsRegistryReplaySession {
        self.manifest_builder.replay_session()
    }

    const fn ensure_usable(&self) -> Result<(), NnsCertifiedRegistryArchiveStorageError> {
        if self.poisoned {
            Err(NnsCertifiedRegistryArchiveStorageError::PublisherPoisoned)
        } else {
            Ok(())
        }
    }
}

///
/// NnsCertifiedRegistryArchiveStorageError
///
/// Typed failures from confined publication or bounded archive restoration.
///

#[derive(Debug, ThisError)]
pub enum NnsCertifiedRegistryArchiveStorageError {
    /// Manifest construction, validation, or replay failed.
    #[error(transparent)]
    Archive(#[from] NnsCertifiedRegistryArchiveError),

    /// A confined filesystem or atomic-write operation failed.
    #[error("certified Registry archive filesystem operation failed: {source}")]
    FileOperation {
        /// Underlying managed-file failure.
        #[source]
        source: CacheFileError,
    },

    /// The archive manifest does not exist at the selected managed location.
    #[error("certified Registry archive manifest is missing at {}", path.display())]
    MissingManifest {
        /// Expected manifest path.
        path: PathBuf,
    },

    /// A report object named by the manifest does not exist.
    #[error(
        "certified Registry archive batch {ordinal} object is missing at {}",
        path.display()
    )]
    MissingBatchObject {
        /// Canonical batch position.
        ordinal: u64,
        /// Expected content-addressed object path.
        path: PathBuf,
    },

    /// A managed archive file exceeds its explicit read or write ceiling.
    #[error(
        "certified Registry archive {kind} at {} is {actual} bytes; maximum is {maximum}",
        path.display()
    )]
    FileLimitExceeded {
        /// Stable file role.
        kind: &'static str,
        /// Managed file path.
        path: PathBuf,
        /// Observed or candidate byte length.
        actual: u64,
        /// Caller-selected ceiling.
        maximum: u64,
    },

    /// A report object's exact byte length differs from its descriptor.
    #[error(
        "certified Registry archive batch {ordinal} object length mismatch at {}: expected {expected}, got {actual}",
        path.display()
    )]
    BatchLengthMismatch {
        /// Canonical batch position.
        ordinal: u64,
        /// Content-addressed object path.
        path: PathBuf,
        /// Descriptor-committed byte length.
        expected: u64,
        /// Actual retained object byte length.
        actual: u64,
    },

    /// A report object's exact SHA-256 differs from its descriptor.
    #[error(
        "certified Registry archive batch {ordinal} object digest mismatch at {}: expected {expected}, got {actual}",
        path.display()
    )]
    BatchDigestMismatch {
        /// Canonical batch position.
        ordinal: u64,
        /// Content-addressed object path.
        path: PathBuf,
        /// Descriptor-committed SHA-256.
        expected: String,
        /// Actual SHA-256.
        actual: String,
    },

    /// The manifest is not exact canonical compact JSON for its schema.
    #[error("certified Registry archive manifest is not canonical compact JSON at {}", path.display())]
    NonCanonicalManifest {
        /// Noncanonical manifest path.
        path: PathBuf,
    },

    /// The manifest could not be decoded from JSON.
    #[error("failed to parse certified Registry archive manifest at {}: {source}", path.display())]
    ParseManifest {
        /// Manifest path.
        path: PathBuf,
        /// JSON decoding failure.
        source: serde_json::Error,
    },

    /// A canonical manifest could not be serialized for publication or comparison.
    #[error("failed to serialize certified Registry archive manifest: {source}")]
    SerializeManifest {
        /// JSON serialization failure.
        source: serde_json::Error,
    },

    /// One retained report object could not be decoded from JSON.
    #[error(
        "failed to parse certified Registry archive batch {ordinal} at {}: {source}",
        path.display()
    )]
    ParseBatchObject {
        /// Canonical batch position.
        ordinal: u64,
        /// Report object path.
        path: PathBuf,
        /// JSON decoding failure.
        source: serde_json::Error,
    },

    /// A retained report's collection timestamp could not recreate its validation request.
    #[error(
        "certified Registry archive batch {ordinal} has invalid fetched_at timestamp {fetched_at:?}"
    )]
    InvalidBatchTimestamp {
        /// Canonical batch position.
        ordinal: u64,
        /// Retained timestamp text.
        fetched_at: String,
    },

    /// Local certificate, witness, chunk, or report authentication failed.
    #[error("certified Registry archive batch {ordinal} authentication failed: {source}")]
    BatchAuthentication {
        /// Canonical batch position.
        ordinal: u64,
        /// Typed retained-evidence failure.
        #[source]
        source: NnsRegistryHostError,
    },

    /// Recomputed authenticated contents do not equal the serialized manifest.
    #[error("certified Registry archive manifest differs from authenticated replay contents")]
    ManifestMismatch,

    /// A prior object-publication failure makes this publisher unsafe to continue.
    #[error("certified Registry archive publisher must be discarded after publication failure")]
    PublisherPoisoned,

    /// Integer file accounting could not be represented safely.
    #[error("certified Registry archive storage accounting overflow")]
    Accounting,
}

/// Return the fixed manifest path beneath a caller-selected archive directory.
#[must_use]
pub fn nns_certified_registry_archive_manifest_path(archive_root: &Path) -> PathBuf {
    archive_root.join(ARCHIVE_MANIFEST_FILE_NAME)
}

/// Load, reauthenticate, replay, and exactly match one confined certified Registry archive.
pub fn load_nns_certified_registry_archive(
    cache_root: &Path,
    archive_root: &Path,
    replay_limits: NnsRegistryReplaySessionLimits,
    storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveStorageError> {
    load_nns_certified_registry_archive_with_authenticator(
        cache_root,
        archive_root,
        replay_limits,
        storage_limits,
        &BuiltInArchiveAuthenticator,
    )
}

///
/// ArchiveBatchAuthenticator
///
/// Internal retained-report authentication seam used by the built-in loader and fixtures.
///

pub(in crate::nns::registry::replay) trait ArchiveBatchAuthenticator {
    /// Qualify one retained report for authenticated replay without a source call.
    fn authenticate<'a>(
        &self,
        request: &NnsCertifiedRegistryDeltaBatchRequest,
        report: &'a NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsAuthenticatedRegistryDeltaBatch<'a>, NnsRegistryHostError>;
}

struct BuiltInArchiveAuthenticator;

impl ArchiveBatchAuthenticator for BuiltInArchiveAuthenticator {
    fn authenticate<'a>(
        &self,
        request: &NnsCertifiedRegistryDeltaBatchRequest,
        report: &'a NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsAuthenticatedRegistryDeltaBatch<'a>, NnsRegistryHostError> {
        reauthenticate_nns_certified_registry_delta_batch(request, report)
    }
}

/// Restore one archive through an explicit local authentication implementation.
pub(in crate::nns::registry::replay) fn load_nns_certified_registry_archive_with_authenticator(
    cache_root: &Path,
    archive_root: &Path,
    replay_limits: NnsRegistryReplaySessionLimits,
    storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
    authenticator: &dyn ArchiveBatchAuthenticator,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveStorageError> {
    let manifest_path = nns_certified_registry_archive_manifest_path(archive_root);
    let manifest_bytes = read_bounded_managed_file(
        cache_root,
        &manifest_path,
        "manifest",
        storage_limits.max_manifest_bytes,
    )?
    .ok_or_else(
        || NnsCertifiedRegistryArchiveStorageError::MissingManifest {
            path: manifest_path.clone(),
        },
    )?;
    let manifest: NnsCertifiedRegistryArchiveManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(
            |source| NnsCertifiedRegistryArchiveStorageError::ParseManifest {
                path: manifest_path.clone(),
                source,
            },
        )?;
    super::validate_nns_certified_registry_archive_manifest(&manifest, storage_limits.archive)?;
    let canonical_manifest = serde_json::to_vec(&manifest)
        .map_err(|source| NnsCertifiedRegistryArchiveStorageError::SerializeManifest { source })?;
    if canonical_manifest != manifest_bytes {
        return Err(
            NnsCertifiedRegistryArchiveStorageError::NonCanonicalManifest {
                path: manifest_path,
            },
        );
    }

    let mut builder =
        NnsCertifiedRegistryArchiveManifestBuilder::new(replay_limits, storage_limits.archive);
    for descriptor in &manifest.batches {
        let object_path = archive_batch_object_path(archive_root, descriptor);
        let object_bytes = read_bounded_managed_file(
            cache_root,
            &object_path,
            "batch object",
            storage_limits.archive.max_batch_report_bytes,
        )?
        .ok_or_else(
            || NnsCertifiedRegistryArchiveStorageError::MissingBatchObject {
                ordinal: descriptor.ordinal,
                path: object_path.clone(),
            },
        )?;
        validate_object_bytes(descriptor, &object_path, &object_bytes)?;
        let report: NnsCertifiedRegistryDeltaBatchReport = serde_json::from_slice(&object_bytes)
            .map_err(
                |source| NnsCertifiedRegistryArchiveStorageError::ParseBatchObject {
                    ordinal: descriptor.ordinal,
                    path: object_path,
                    source,
                },
            )?;
        drop(object_bytes);
        let now_unix_secs = parse_utc_timestamp_secs(&report.fetched_at).ok_or_else(|| {
            NnsCertifiedRegistryArchiveStorageError::InvalidBatchTimestamp {
                ordinal: descriptor.ordinal,
                fetched_at: report.fetched_at.clone(),
            }
        })?;
        let request = NnsCertifiedRegistryDeltaBatchRequest::new(
            &report.network,
            &report.source_endpoint,
            report.requested_version,
            now_unix_secs,
        );
        let authenticated = authenticator
            .authenticate(&request, &report)
            .map_err(
                |source| NnsCertifiedRegistryArchiveStorageError::BatchAuthentication {
                    ordinal: descriptor.ordinal,
                    source,
                },
            )?;
        builder.apply_batch(&authenticated)?;
    }
    let (recomputed, replay_session) = builder.finish()?;
    if recomputed != manifest {
        return Err(NnsCertifiedRegistryArchiveStorageError::ManifestMismatch);
    }
    Ok(NnsAuthenticatedRegistryArchive {
        manifest,
        replay_session,
    })
}

fn write_report_object(
    cache_root: &Path,
    archive_root: &Path,
    descriptor: &NnsCertifiedRegistryArchiveBatchDescriptor,
    report: &NnsCertifiedRegistryDeltaBatchReport,
) -> Result<(), NnsCertifiedRegistryArchiveStorageError> {
    let path = archive_batch_object_path(archive_root, descriptor);
    write_managed_file_atomically(cache_root, &path, |file| {
        let mut writer = DigestingWriter::new(file);
        serde_json::to_writer(&mut writer, report).map_err(json_io_error)?;
        let (bytes, digest) = writer.finish();
        if bytes != descriptor.report_bytes || hex_bytes(&digest) != descriptor.report_sha256 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "canonical report encoding changed after manifest admission",
            ));
        }
        Ok(())
    })
    .map_err(file_operation)
}

fn write_manifest(
    cache_root: &Path,
    archive_root: &Path,
    manifest: &NnsCertifiedRegistryArchiveManifest,
    max_manifest_bytes: u64,
) -> Result<(), NnsCertifiedRegistryArchiveStorageError> {
    let path = nns_certified_registry_archive_manifest_path(archive_root);
    let manifest_bytes = canonical_serialized_len(manifest)?;
    if manifest_bytes > max_manifest_bytes {
        return Err(NnsCertifiedRegistryArchiveStorageError::FileLimitExceeded {
            kind: "manifest",
            path,
            actual: manifest_bytes,
            maximum: max_manifest_bytes,
        });
    }
    write_managed_file_atomically(cache_root, &path, |file| {
        let mut writer = BoundedWriter::new(file, max_manifest_bytes);
        serde_json::to_writer(&mut writer, manifest).map_err(json_io_error)
    })
    .map_err(file_operation)
}

fn read_bounded_managed_file(
    cache_root: &Path,
    path: &Path,
    kind: &'static str,
    maximum: u64,
) -> Result<Option<Vec<u8>>, NnsCertifiedRegistryArchiveStorageError> {
    let Some(mut file) = open_managed_file(cache_root, path).map_err(file_operation)? else {
        return Ok(None);
    };
    let metadata_length = file
        .metadata()
        .map_err(|source| {
            file_operation(CacheFileError::OpenManagedPath {
                root: cache_root.to_path_buf(),
                path: path.to_path_buf(),
                source,
            })
        })?
        .len();
    if metadata_length > maximum {
        return Err(NnsCertifiedRegistryArchiveStorageError::FileLimitExceeded {
            kind,
            path: path.to_path_buf(),
            actual: metadata_length,
            maximum,
        });
    }
    let capacity = usize::try_from(metadata_length)
        .map_err(|_| NnsCertifiedRegistryArchiveStorageError::Accounting)?;
    let mut bytes = Vec::with_capacity(capacity);
    Read::by_ref(&mut file)
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| {
            file_operation(CacheFileError::OpenManagedPath {
                root: cache_root.to_path_buf(),
                path: path.to_path_buf(),
                source,
            })
        })?;
    let actual = u64::try_from(bytes.len())
        .map_err(|_| NnsCertifiedRegistryArchiveStorageError::Accounting)?;
    if actual > maximum {
        return Err(NnsCertifiedRegistryArchiveStorageError::FileLimitExceeded {
            kind,
            path: path.to_path_buf(),
            actual,
            maximum,
        });
    }
    Ok(Some(bytes))
}

fn validate_object_bytes(
    descriptor: &NnsCertifiedRegistryArchiveBatchDescriptor,
    path: &Path,
    bytes: &[u8],
) -> Result<(), NnsCertifiedRegistryArchiveStorageError> {
    let actual_length = u64::try_from(bytes.len())
        .map_err(|_| NnsCertifiedRegistryArchiveStorageError::Accounting)?;
    if actual_length != descriptor.report_bytes {
        return Err(
            NnsCertifiedRegistryArchiveStorageError::BatchLengthMismatch {
                ordinal: descriptor.ordinal,
                path: path.to_path_buf(),
                expected: descriptor.report_bytes,
                actual: actual_length,
            },
        );
    }
    let actual_digest = hex_bytes(&Sha256::digest(bytes));
    if actual_digest != descriptor.report_sha256 {
        return Err(
            NnsCertifiedRegistryArchiveStorageError::BatchDigestMismatch {
                ordinal: descriptor.ordinal,
                path: path.to_path_buf(),
                expected: descriptor.report_sha256.clone(),
                actual: actual_digest,
            },
        );
    }
    Ok(())
}

fn archive_batch_object_path(
    archive_root: &Path,
    descriptor: &NnsCertifiedRegistryArchiveBatchDescriptor,
) -> PathBuf {
    archive_root
        .join(ARCHIVE_OBJECTS_DIRECTORY_NAME)
        .join(format!("{}.json", descriptor.report_sha256))
}

const fn file_operation(source: CacheFileError) -> NnsCertifiedRegistryArchiveStorageError {
    NnsCertifiedRegistryArchiveStorageError::FileOperation { source }
}

fn canonical_serialized_len(
    value: &impl Serialize,
) -> Result<u64, NnsCertifiedRegistryArchiveStorageError> {
    let mut writer = CountingWriter::default();
    serde_json::to_writer(&mut writer, value)
        .map_err(|source| NnsCertifiedRegistryArchiveStorageError::SerializeManifest { source })?;
    Ok(writer.bytes)
}

fn json_io_error(error: serde_json::Error) -> io::Error {
    match error.io_error_kind() {
        Some(kind) => io::Error::new(kind, error),
        None => io::Error::other(error),
    }
}

#[derive(Default)]
struct CountingWriter {
    bytes: u64,
}

impl Write for CountingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let length = u64::try_from(buffer.len())
            .map_err(|_| io::Error::other("archive write length exceeds u64"))?;
        self.bytes = self
            .bytes
            .checked_add(length)
            .ok_or_else(|| io::Error::other("archive write length exceeds u64"))?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct DigestingWriter<'a> {
    writer: &'a mut cap_std::fs::File,
    hasher: Sha256,
    bytes: u64,
}

impl<'a> DigestingWriter<'a> {
    fn new(writer: &'a mut cap_std::fs::File) -> Self {
        Self {
            writer,
            hasher: Sha256::new(),
            bytes: 0,
        }
    }

    fn finish(self) -> (u64, [u8; 32]) {
        (self.bytes, self.hasher.finalize().into())
    }
}

impl Write for DigestingWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.writer.write_all(buffer)?;
        let length = u64::try_from(buffer.len())
            .map_err(|_| io::Error::other("archive write length exceeds u64"))?;
        self.bytes = self
            .bytes
            .checked_add(length)
            .ok_or_else(|| io::Error::other("archive write length exceeds u64"))?;
        self.hasher.update(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

struct BoundedWriter<'a> {
    writer: &'a mut cap_std::fs::File,
    maximum: u64,
    bytes: u64,
}

impl<'a> BoundedWriter<'a> {
    const fn new(writer: &'a mut cap_std::fs::File, maximum: u64) -> Self {
        Self {
            writer,
            maximum,
            bytes: 0,
        }
    }
}

impl Write for BoundedWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let length = u64::try_from(buffer.len())
            .map_err(|_| io::Error::other("archive write length exceeds u64"))?;
        let candidate = self
            .bytes
            .checked_add(length)
            .ok_or_else(|| io::Error::other("archive write length exceeds u64"))?;
        if candidate > self.maximum {
            return Err(io::Error::new(
                io::ErrorKind::FileTooLarge,
                "archive file exceeds its explicit byte ceiling",
            ));
        }
        self.writer.write_all(buffer)?;
        self.bytes = candidate;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
