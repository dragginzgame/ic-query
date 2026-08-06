//! Module: nns::registry::replay::projection
//!
//! Responsibility: project complete replay state and promote reauthenticated archives to catalogs.
//! Does not own: authentication establishment, archive storage, cache policy, or serialization.
//! Boundary: certified authority borrows its archive and cannot outlive the qualifying evidence.

use super::super::NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION;
use super::{
    NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION,
    NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION, NnsAuthenticatedRegistryArchive,
    NnsRegistryReplaySession,
};
use crate::{
    hex::hex_bytes,
    ic_registry::{
        ROUTING_TABLE_KEY, SUBNET_LIST_KEY, proto::RoutingTable, proto::SubnetListRecord,
        proto::SubnetRecord, routing_ranges_from_table, subnet_info_from_record, subnet_record_key,
    },
    subnet_catalog::{
        CATALOG_SCHEMA_VERSION, CatalogAssurance, CatalogError, CatalogValidationContext,
        CertifiedRegistryCatalogEvidence, MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID,
        RawSubnetCatalog, RoutingRange, SubnetCatalogProvenance, SubnetInfo,
        ValidatedSubnetCatalog, canonicalize_subnet_catalog_content, format_utc_timestamp_secs,
    },
};
use candid::Principal;
use prost::Message;
use thiserror::Error as ThisError;

const NANOS_PER_SECOND: u128 = 1_000_000_000;

///
/// NnsRegistrySubnetCatalogProjection
///
/// Canonical Subnet Catalog rows derived from one complete exact-target replay session.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsRegistrySubnetCatalogProjection<'a> {
    session: &'a NnsRegistryReplaySession,
    registry_version: u64,
    subnets: Vec<SubnetInfo>,
    routing_ranges: Vec<RoutingRange>,
}

impl<'a> NnsRegistrySubnetCatalogProjection<'a> {
    /// Return the complete replay session that qualifies this projection.
    #[must_use]
    pub const fn replay_session(&self) -> &'a NnsRegistryReplaySession {
        self.session
    }

    /// Return the exact Registry version shared by every projected record.
    #[must_use]
    pub const fn registry_version(&self) -> u64 {
        self.registry_version
    }

    /// Return canonical Subnet rows classified through the existing catalog policy.
    #[must_use]
    pub fn subnets(&self) -> &[SubnetInfo] {
        &self.subnets
    }

    /// Return canonical inclusive canister routing ranges.
    #[must_use]
    pub fn routing_ranges(&self) -> &[RoutingRange] {
        &self.routing_ranges
    }
}

///
/// NnsCertifiedSubnetCatalogVersionPolicy
///
/// Caller-selected treatment of a pinned target superseded during archive collection.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NnsCertifiedSubnetCatalogVersionPolicy {
    /// Accept the authenticated exact target even when a later batch observed a newer Registry.
    AllowHistoricalTarget,
    /// Require the selected target to equal the newest Registry version observed by every batch.
    RequireLatestObserved,
}

///
/// NnsCertifiedSubnetCatalogProjectionRequest
///
/// Caller-owned identity, time, and maximum certificate-age policy for one promotion.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsCertifiedSubnetCatalogProjectionRequest {
    /// Required catalog identity and future-skew policy.
    pub validation: CatalogValidationContext,
    /// Maximum accepted age of the latest certificate at the observation time.
    pub maximum_certificate_age_seconds: u64,
    /// Required relation between the pinned target and later certified version observations.
    pub version_policy: NnsCertifiedSubnetCatalogVersionPolicy,
}

impl NnsCertifiedSubnetCatalogProjectionRequest {
    /// Create one explicit certified catalog projection policy without a hidden age default.
    #[must_use]
    pub const fn new(
        validation: CatalogValidationContext,
        maximum_certificate_age_seconds: u64,
        version_policy: NnsCertifiedSubnetCatalogVersionPolicy,
    ) -> Self {
        Self {
            validation,
            maximum_certificate_age_seconds,
            version_policy,
        }
    }
}

///
/// NnsCertifiedSubnetCatalogFreshness
///
/// Exact certificate-age decision retained with one promoted catalog authority.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NnsCertifiedSubnetCatalogFreshness {
    /// Caller-supplied observation time in Unix seconds.
    pub observation_time_unix_seconds: u64,
    /// Latest authenticated archive certificate time in Unix nanoseconds.
    pub latest_certificate_time_nanos: u64,
    /// Exact nonnegative age at the caller's observation time.
    pub certificate_age_nanos: u128,
    /// Caller-supplied maximum accepted certificate age in seconds.
    pub maximum_certificate_age_seconds: u64,
    /// Exact Registry target represented by the projected catalog.
    pub selected_registry_version: u64,
    /// Newest certified Registry version observed anywhere in the archive.
    pub maximum_observed_certified_registry_version: u64,
    /// Caller-selected treatment of a target superseded during collection.
    pub version_policy: NnsCertifiedSubnetCatalogVersionPolicy,
}

///
/// NnsCertifiedSubnetCatalogAuthority
///
/// Validated certified catalog kept attached to the archive that establishes its authority.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsCertifiedSubnetCatalogAuthority<'a> {
    archive: &'a NnsAuthenticatedRegistryArchive,
    catalog: ValidatedSubnetCatalog,
    freshness: NnsCertifiedSubnetCatalogFreshness,
}

impl<'a> NnsCertifiedSubnetCatalogAuthority<'a> {
    /// Return the fully reauthenticated archive qualifying this catalog.
    #[must_use]
    pub const fn archive(&self) -> &'a NnsAuthenticatedRegistryArchive {
        self.archive
    }

    /// Return the validated catalog promoted from the attached archive.
    #[must_use]
    pub const fn catalog(&self) -> &ValidatedSubnetCatalog {
        &self.catalog
    }

    /// Return the exact caller-policy freshness decision qualifying this authority.
    #[must_use]
    pub const fn freshness(&self) -> NnsCertifiedSubnetCatalogFreshness {
        self.freshness
    }
}

///
/// NnsRegistrySubnetCatalogProjectionError
///
/// Typed failures returned before replay state is exposed as catalog content.
///

#[derive(Debug, ThisError)]
pub enum NnsRegistrySubnetCatalogProjectionError {
    /// The replay session has not reached its pinned exact target.
    #[error(
        "Registry replay session is incomplete: selected version {selected_version:?}, through version {through_version}"
    )]
    IncompleteSession {
        /// Exact target selected from the first admitted report, when available.
        selected_version: Option<u64>,
        /// Last Registry version currently reconstructed.
        through_version: u64,
    },

    /// Registry version zero cannot identify a catalog snapshot.
    #[error("Registry replay selected version must be greater than zero for catalog projection")]
    InvalidRegistryVersion,

    /// Complete replay state does not contain one record required by the catalog.
    #[error("complete Registry replay state is missing required key {key:?}")]
    MissingRequiredRegistryKey {
        /// Exact raw Registry key interpreted as canonical UTF-8 text.
        key: String,
    },

    /// One required replayed Registry record could not be interpreted.
    #[error("replayed Registry key {key:?} is not a valid {message}: {reason}")]
    InvalidRegistryRecord {
        /// Exact Registry key containing the invalid value.
        key: String,
        /// Expected Registry record type.
        message: &'static str,
        /// Deterministic decoding or structural failure.
        reason: String,
    },

    /// Canonical catalog classification or routing validation failed.
    #[error(transparent)]
    Catalog(#[from] CatalogError),

    /// The archive manifest and its sealed replay session disagree.
    #[error(
        "authenticated Registry archive mismatch in {field}: manifest={manifest_value}, replay={replay_value}"
    )]
    ArchiveEvidenceMismatch {
        /// Exact archive or replay field that disagreed.
        field: &'static str,
        /// Deterministic manifest-side value.
        manifest_value: String,
        /// Deterministic replay-side or implementation-side value.
        replay_value: String,
    },

    /// The latest authenticated certificate is older than the caller permits.
    #[error(
        "authenticated Registry archive certificate is stale: latest_certificate_time_nanos={latest_certificate_time_nanos}, observation_time_unix_seconds={observation_time_unix_seconds}, certificate_age_nanos={certificate_age_nanos}, maximum_certificate_age_seconds={maximum_certificate_age_seconds}"
    )]
    StaleArchiveCertificate {
        /// Latest authenticated certificate time retained by the archive.
        latest_certificate_time_nanos: u64,
        /// Caller-supplied time at which freshness was evaluated.
        observation_time_unix_seconds: u64,
        /// Exact nonnegative certificate age in nanoseconds.
        certificate_age_nanos: u128,
        /// Caller-supplied maximum accepted age.
        maximum_certificate_age_seconds: u64,
    },

    /// A later archive batch certified a newer Registry version than the selected target.
    #[error(
        "authenticated Registry archive target is superseded: selected_registry_version={selected_registry_version}, maximum_observed_certified_registry_version={maximum_observed_certified_registry_version}"
    )]
    SupersededArchiveTarget {
        /// Exact Registry version reconstructed by the archive.
        selected_registry_version: u64,
        /// Newest Registry version certified by any retained batch.
        maximum_observed_certified_registry_version: u64,
    },
}

/// Project a complete exact-target replay session into canonical Subnet Catalog content.
///
/// The returned value borrows `session`, keeping the projected rows attached to
/// their replay provenance. It is not a serialized mirror, a
/// `ValidatedSubnetCatalog`, or a `CatalogAssurance::Certified` promotion.
pub fn project_nns_registry_subnet_catalog(
    session: &NnsRegistryReplaySession,
) -> Result<NnsRegistrySubnetCatalogProjection<'_>, NnsRegistrySubnetCatalogProjectionError> {
    let selected_version = session.selected_version();
    let (true, Some(registry_version)) = (
        session.is_complete() && session.complete_state_digest().is_some(),
        selected_version,
    ) else {
        return Err(NnsRegistrySubnetCatalogProjectionError::IncompleteSession {
            selected_version,
            through_version: session.state().through_version(),
        });
    };
    if registry_version == 0 {
        return Err(NnsRegistrySubnetCatalogProjectionError::InvalidRegistryVersion);
    }

    let state = session.state();
    let subnet_list =
        decode_required_record::<SubnetListRecord>(state, SUBNET_LIST_KEY, "SubnetListRecord")?;
    let routing_table =
        decode_required_record::<RoutingTable>(state, ROUTING_TABLE_KEY, "RoutingTable")?;
    let mut subnets = Vec::with_capacity(subnet_list.subnets.len());
    for raw_subnet_principal in subnet_list.subnets {
        let subnet_principal = Principal::try_from_slice(&raw_subnet_principal)
            .map(|principal| principal.to_text())
            .map_err(|error| invalid_record(SUBNET_LIST_KEY, "SubnetListRecord", error))?;
        let record_key = subnet_record_key(&subnet_principal);
        let record = decode_required_record::<SubnetRecord>(state, &record_key, "SubnetRecord")?;
        subnets.push(subnet_info_from_record(&subnet_principal, &record));
    }
    let mut routing_ranges = routing_ranges_from_table(&routing_table)
        .map_err(|error| invalid_record(ROUTING_TABLE_KEY, "RoutingTable", error))?;
    canonicalize_subnet_catalog_content(&mut subnets, &mut routing_ranges)?;

    Ok(NnsRegistrySubnetCatalogProjection {
        session,
        registry_version,
        subnets,
        routing_ranges,
    })
}

/// Promote one fully reauthenticated Registry archive into certified catalog authority.
///
/// The returned capability borrows `archive`; serializing its raw catalog does not preserve
/// certified authority, and ordinary `ValidatedSubnetCatalog::try_from_raw` validation will
/// continue to reject the serialized `Certified` claim.
pub fn project_nns_certified_subnet_catalog<'a>(
    archive: &'a NnsAuthenticatedRegistryArchive,
    request: &NnsCertifiedSubnetCatalogProjectionRequest,
) -> Result<NnsCertifiedSubnetCatalogAuthority<'a>, NnsRegistrySubnetCatalogProjectionError> {
    validate_archive_replay_alignment(archive)?;
    let freshness = validate_archive_certificate_freshness(archive, request)?;
    let projection =
        project_nns_registry_subnet_catalog(archive.replay_session().replay_session())?;
    let manifest = archive.manifest();
    require_archive_match(
        "projected_registry_version",
        manifest.selected_version,
        projection.registry_version(),
    )?;
    let provenance = SubnetCatalogProvenance {
        network: manifest.network.clone(),
        registry_canister_id: manifest.registry_canister_id.clone(),
        registry_version: manifest.selected_version,
        assurance: CatalogAssurance::Certified,
        source_endpoints: manifest.source_endpoints.clone(),
        agreement_digest: None,
        registry_query_call_count: manifest.query_call_count,
        fetched_at: format_utc_timestamp_secs(
            manifest.maximum_certificate_time_nanos / 1_000_000_000,
        ),
        certified_registry: Some(CertifiedRegistryCatalogEvidence {
            archive_manifest_schema_version: manifest.schema_version,
            delta_report_schema_version: manifest.delta_report_schema_version,
            replay_provenance_schema_version: manifest.replay_provenance_schema_version,
            root_key_digest: manifest.root_key_digest.clone(),
            evidence_chain_digest: manifest.evidence_chain_digest.clone(),
            complete_state_digest: manifest.complete_state_digest.clone(),
            minimum_certificate_time_nanos: manifest.minimum_certificate_time_nanos,
            maximum_certificate_time_nanos: manifest.maximum_certificate_time_nanos,
        }),
        fetched_by: "ic-query".to_string(),
        collector_version: env!("CARGO_PKG_VERSION").to_string(),
        classification_schema_version: crate::subnet_catalog::CLASSIFICATION_SCHEMA_VERSION,
        classification_policy_digest: String::new(),
        resolver_schema_version: crate::subnet_catalog::RESOLVER_SCHEMA_VERSION,
        resolver_backend: String::new(),
    };
    let mut raw = RawSubnetCatalog {
        catalog_schema_version: CATALOG_SCHEMA_VERSION,
        provenance,
        catalog_digest: String::new(),
        subnets: projection.subnets,
        routing_ranges: projection.routing_ranges,
    };
    raw.canonicalize_and_seal()?;
    let catalog =
        ValidatedSubnetCatalog::try_from_authenticated_archive(raw, &request.validation, archive)?;
    Ok(NnsCertifiedSubnetCatalogAuthority {
        archive,
        catalog,
        freshness,
    })
}

fn validate_archive_certificate_freshness(
    archive: &NnsAuthenticatedRegistryArchive,
    request: &NnsCertifiedSubnetCatalogProjectionRequest,
) -> Result<NnsCertifiedSubnetCatalogFreshness, NnsRegistrySubnetCatalogProjectionError> {
    let latest_certificate_time_nanos = archive.manifest().maximum_certificate_time_nanos;
    let selected_registry_version = archive.manifest().selected_version;
    let maximum_observed_certified_registry_version = archive
        .manifest()
        .batches
        .iter()
        .map(|batch| batch.certified_latest_version)
        .max()
        .unwrap_or(selected_registry_version);
    if request.version_policy == NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved
        && maximum_observed_certified_registry_version != selected_registry_version
    {
        return Err(
            NnsRegistrySubnetCatalogProjectionError::SupersededArchiveTarget {
                selected_registry_version,
                maximum_observed_certified_registry_version,
            },
        );
    }
    let observation_time_nanos = u128::from(request.validation.now_unix_secs) * NANOS_PER_SECOND;
    let certificate_age_nanos =
        observation_time_nanos.saturating_sub(u128::from(latest_certificate_time_nanos));
    let maximum_certificate_age_nanos =
        u128::from(request.maximum_certificate_age_seconds) * NANOS_PER_SECOND;
    if certificate_age_nanos > maximum_certificate_age_nanos {
        return Err(
            NnsRegistrySubnetCatalogProjectionError::StaleArchiveCertificate {
                latest_certificate_time_nanos,
                observation_time_unix_seconds: request.validation.now_unix_secs,
                certificate_age_nanos,
                maximum_certificate_age_seconds: request.maximum_certificate_age_seconds,
            },
        );
    }
    Ok(NnsCertifiedSubnetCatalogFreshness {
        observation_time_unix_seconds: request.validation.now_unix_secs,
        latest_certificate_time_nanos,
        certificate_age_nanos,
        maximum_certificate_age_seconds: request.maximum_certificate_age_seconds,
        selected_registry_version,
        maximum_observed_certified_registry_version,
        version_policy: request.version_policy,
    })
}

fn validate_archive_replay_alignment(
    archive: &NnsAuthenticatedRegistryArchive,
) -> Result<(), NnsRegistrySubnetCatalogProjectionError> {
    let manifest = archive.manifest();
    let session = archive.replay_session().replay_session();
    require_archive_match(
        "archive_manifest_schema_version",
        manifest.schema_version,
        NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION,
    )?;
    require_archive_match(
        "delta_report_schema_version",
        manifest.delta_report_schema_version,
        NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION,
    )?;
    require_archive_match(
        "replay_provenance_schema_version",
        manifest.replay_provenance_schema_version,
        NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION,
    )?;
    require_archive_match("network", manifest.network.as_str(), MAINNET_NETWORK)?;
    require_archive_match(
        "registry_canister_id",
        manifest.registry_canister_id.as_str(),
        MAINNET_REGISTRY_CANISTER_ID,
    )?;
    require_archive_match(
        "selected_version",
        Some(manifest.selected_version),
        session.selected_version(),
    )?;
    require_archive_match(
        "through_version",
        manifest.selected_version,
        session.state().through_version(),
    )?;
    require_archive_match("batch_count", manifest.batch_count, session.batch_count())?;
    require_archive_match(
        "query_call_count",
        manifest.query_call_count,
        session.query_call_count(),
    )?;
    require_archive_match(
        "response_bytes",
        manifest.response_bytes,
        session.response_bytes(),
    )?;
    require_archive_match(
        "applied_mutation_count",
        manifest.applied_mutation_count,
        session.applied_mutation_count(),
    )?;
    require_archive_match(
        "root_key_digest",
        Some(manifest.root_key_digest.as_str()),
        session.root_key_digest(),
    )?;
    require_archive_match(
        "evidence_chain_digest",
        Some(manifest.evidence_chain_digest.clone()),
        session
            .evidence_chain_digest()
            .map(|digest| hex_bytes(&digest)),
    )?;
    require_archive_match(
        "complete_state_digest",
        Some(manifest.complete_state_digest.clone()),
        session
            .complete_state_digest()
            .map(|digest| hex_bytes(&digest)),
    )?;
    require_archive_match(
        "minimum_certificate_time_nanos",
        Some(manifest.minimum_certificate_time_nanos),
        session.minimum_certificate_time_nanos(),
    )?;
    require_archive_match(
        "maximum_certificate_time_nanos",
        Some(manifest.maximum_certificate_time_nanos),
        session.maximum_certificate_time_nanos(),
    )?;
    let replay_endpoints = session.source_endpoints().collect::<Vec<_>>();
    let manifest_endpoints = manifest
        .source_endpoints
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    require_archive_match("source_endpoints", manifest_endpoints, replay_endpoints)
}

fn require_archive_match<T>(
    field: &'static str,
    manifest_value: T,
    replay_value: T,
) -> Result<(), NnsRegistrySubnetCatalogProjectionError>
where
    T: std::fmt::Debug + PartialEq,
{
    if manifest_value != replay_value {
        return Err(
            NnsRegistrySubnetCatalogProjectionError::ArchiveEvidenceMismatch {
                field,
                manifest_value: format!("{manifest_value:?}"),
                replay_value: format!("{replay_value:?}"),
            },
        );
    }
    Ok(())
}

fn decode_required_record<M>(
    state: &super::NnsRegistryReplayState,
    key: &str,
    message: &'static str,
) -> Result<M, NnsRegistrySubnetCatalogProjectionError>
where
    M: Message + Default,
{
    let value = state.get(key.as_bytes()).ok_or_else(|| {
        NnsRegistrySubnetCatalogProjectionError::MissingRequiredRegistryKey {
            key: key.to_string(),
        }
    })?;
    M::decode(value.value()).map_err(|error| invalid_record(key, message, error))
}

fn invalid_record(
    key: &str,
    message: &'static str,
    error: impl ToString,
) -> NnsRegistrySubnetCatalogProjectionError {
    NnsRegistrySubnetCatalogProjectionError::InvalidRegistryRecord {
        key: key.to_string(),
        message,
        reason: error.to_string(),
    }
}
