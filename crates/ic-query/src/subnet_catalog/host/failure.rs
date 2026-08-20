//! Module: subnet_catalog::host::failure
//!
//! Responsibility: retain typed failure provenance across Subnet Catalog load layers.
//! Does not own: cache policy execution, Registry transport, or report rendering.
//! Boundary: combines request, stage, cache, Registry, subject, and source-error evidence.

use super::{
    CatalogSourceSelection, SubnetCatalogErrorCategory, SubnetCatalogErrorCode,
    SubnetCatalogHostError, SubnetCatalogLoadRequest, SubnetCatalogRetryability,
};
use crate::subnet_catalog::{CatalogAssurance, CatalogError, RoutingRange};
use candid::Principal;
use std::{error::Error, fmt, path::PathBuf};

///
/// SubnetCatalogLoadFailureRequest
///
/// Request identity and authority policy retained on a detailed load failure.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogLoadFailureRequest {
    /// Requested network identity.
    pub network: String,
    /// Selected source when the policy authorized a live refresh.
    pub source: Option<CatalogSourceSelection>,
    /// Minimum assurance requested by the caller.
    pub minimum_assurance: CatalogAssurance,
}

impl SubnetCatalogLoadFailureRequest {
    pub(super) fn from_load_request(request: &SubnetCatalogLoadRequest) -> Self {
        Self {
            network: request.cache.network.clone(),
            source: request.policy.source().cloned(),
            minimum_assurance: request.minimum_assurance,
        }
    }
}

///
/// SubnetCatalogLoadStage
///
/// Exact operation stage at which a detailed Subnet Catalog load failed.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubnetCatalogLoadStage {
    /// The request failed before a cache or source operation was selected.
    RequestValidation,
    /// A caller-requested cache-only load failed.
    CacheOnlyLoad,
    /// A normal cache lookup failed for a reason other than absence or rejection.
    CacheLookup,
    /// The selected cache was absent and no later operation supplied a result.
    CacheAbsence,
    /// Cache content was present but rejected.
    CacheRejection,
    /// A forced-refresh policy bypassed the cache but could not start collection.
    CacheBypass,
    /// Refresh preflight failed after a refresh path was selected.
    RefreshAttempted,
    /// Live refresh or publication failed.
    RefreshFailed,
    /// Refresh completed, but loading the newly published cache failed.
    PostRefreshCacheLoadFailed,
    /// The synchronous adapter could not execute the async load.
    RuntimeAdapter,
}

impl SubnetCatalogLoadStage {
    /// Return the stable snake-case stage.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestValidation => "request_validation",
            Self::CacheOnlyLoad => "cache_only_load",
            Self::CacheLookup => "cache_lookup",
            Self::CacheAbsence => "cache_absence",
            Self::CacheRejection => "cache_rejection",
            Self::CacheBypass => "cache_bypass",
            Self::RefreshAttempted => "refresh_attempted",
            Self::RefreshFailed => "refresh_failed",
            Self::PostRefreshCacheLoadFailed => "post_refresh_cache_load_failed",
            Self::RuntimeAdapter => "runtime_adapter",
        }
    }
}

///
/// SubnetCatalogRefreshTrigger
///
/// Cache-policy fact that caused a Subnet Catalog refresh path to be selected.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubnetCatalogRefreshTrigger {
    /// No cache file existed.
    Missing,
    /// Existing cache content was rejected.
    Rejected,
    /// Existing valid cache content exceeded the caller's age policy.
    Stale,
    /// The caller explicitly bypassed cache reuse.
    Forced,
}

impl SubnetCatalogRefreshTrigger {
    /// Return the stable snake-case trigger.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Rejected => "rejected",
            Self::Stale => "stale",
            Self::Forced => "forced",
        }
    }
}

///
/// SubnetCatalogFailureCacheDisposition
///
/// Exact failure-side cache state or attempted cache action for one detailed load.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubnetCatalogFailureCacheDisposition {
    /// No cache operation was reached.
    NotExamined,
    /// The request authorized only a local cache load.
    CacheOnly,
    /// A forced refresh deliberately bypassed cache reuse.
    CacheBypassed,
    /// The selected cache was absent.
    CacheMissing,
    /// Existing cache content was rejected.
    CacheRejected,
    /// Cache IO failed before content could be accepted or rejected.
    CacheReadFailed,
    /// A refresh path was selected but failed during preflight.
    RefreshAttempted(SubnetCatalogRefreshTrigger),
    /// Refresh collection or publication failed.
    RefreshFailed(SubnetCatalogRefreshTrigger),
    /// Refresh succeeded, but the resulting cache could not be loaded.
    PostRefreshLoadFailed(SubnetCatalogRefreshTrigger),
}

///
/// SubnetCatalogRegistryRecordKind
///
/// Registry method or record family involved in a Subnet Catalog failure.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubnetCatalogRegistryRecordKind {
    /// The Registry Subnet list record.
    SubnetList,
    /// The Registry routing table record.
    RoutingTable,
    /// One exact Registry Subnet record.
    SubnetRecord,
}

impl SubnetCatalogRegistryRecordKind {
    /// Return the stable snake-case kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubnetList => "subnet_list",
            Self::RoutingTable => "routing_table",
            Self::SubnetRecord => "subnet_record",
        }
    }
}

///
/// SubnetCatalogRegistryRecordSubject
///
/// Typed Registry record identity retained for a Subnet Catalog failure.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogRegistryRecordSubject {
    /// Registry record family.
    pub kind: SubnetCatalogRegistryRecordKind,
    /// Exact Registry key used by `get_value`.
    pub key: String,
    /// Exact Subnet principal for a Subnet-record operation.
    pub subnet: Option<Principal>,
}

impl SubnetCatalogRegistryRecordSubject {
    #[must_use]
    pub(crate) fn keyed(kind: SubnetCatalogRegistryRecordKind, key: impl Into<String>) -> Self {
        Self {
            kind,
            key: key.into(),
            subnet: None,
        }
    }

    #[must_use]
    pub(crate) fn subnet_record(key: impl Into<String>, subnet: Principal) -> Self {
        Self {
            kind: SubnetCatalogRegistryRecordKind::SubnetRecord,
            key: key.into(),
            subnet: Some(subnet),
        }
    }
}

///
/// SubnetCatalogField
///
/// Typed field identity known by the Subnet Catalog collector or validator.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubnetCatalogField {
    /// One principal entry in the Registry Subnet list.
    SubnetListEntry,
    /// The range object in one routing-table entry.
    RoutingTableRange,
    /// The target Subnet id in one routing-table entry.
    RoutingTableSubnetId,
    /// The starting canister id in one routing range.
    RoutingRangeStart,
    /// The ending canister id in one routing range.
    RoutingRangeEnd,
    /// Catalog network provenance.
    Network,
    /// Catalog Registry-canister provenance.
    RegistryCanister,
    /// Catalog Registry-version provenance.
    RegistryVersion,
    /// Catalog source endpoint provenance.
    SourceEndpoint,
    /// One Subnet principal field.
    SubnetPrincipal,
    /// Catalog collection timestamp provenance.
    CollectionTimestamp,
    /// Catalog classification evidence.
    Classification,
    /// Catalog agreement digest evidence.
    AgreementDigest,
    /// Catalog authority digest evidence.
    CatalogDigest,
    /// Catalog provenance not narrowed to another field.
    Provenance,
}

///
/// SubnetCatalogSubject
///
/// Typed offending identity retained when the failing layer knows it.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubnetCatalogSubject {
    /// Requested network identity.
    Network(String),
    /// Exact selected or failing endpoint.
    Endpoint(String),
    /// Exact managed cache path.
    CachePath(PathBuf),
    /// The Registry latest-version query, before an exact version is known.
    RegistryLatestVersion,
    /// Registry record, key, and optional Subnet identity.
    RegistryRecord(SubnetCatalogRegistryRecordSubject),
    /// Exact Subnet principal, optionally narrowed to one typed field.
    Subnet {
        /// Offending Subnet.
        subnet: Principal,
        /// Narrower field identity when known.
        field: Option<SubnetCatalogField>,
    },
    /// One indexed Registry routing-table entry and its offending field when known.
    RegistryRoutingTableEntry {
        /// Zero-based position in the pinned routing table.
        index: usize,
        /// Narrower field identity when known.
        field: Option<SubnetCatalogField>,
    },
    /// One complete routing-range value from cache or source validation.
    RoutingRange {
        /// Offending routing range.
        range: RoutingRange,
        /// Narrower field identity when known.
        field: Option<SubnetCatalogField>,
    },
    /// A typed field not owned by one narrower subject.
    Field(SubnetCatalogField),
}

///
/// SubnetCatalogSourceFailure
///
/// Detailed failure returned by a caller-supplied Subnet Catalog source.
///

#[derive(Debug)]
pub struct SubnetCatalogSourceFailure {
    /// Exact pinned Registry version when collection progressed far enough to know it.
    pub registry_version: Option<u64>,
    /// Typed offending identity when known.
    pub subject: Option<SubnetCatalogSubject>,
    /// Original host error.
    pub source: SubnetCatalogHostError,
}

impl SubnetCatalogSourceFailure {
    /// Retain source-level Registry and subject provenance.
    #[must_use]
    pub const fn new(
        registry_version: Option<u64>,
        subject: Option<SubnetCatalogSubject>,
        source: SubnetCatalogHostError,
    ) -> Self {
        Self {
            registry_version,
            subject,
            source,
        }
    }

    /// Wrap a source that cannot provide narrower provenance.
    #[must_use]
    pub const fn from_source(source: SubnetCatalogHostError) -> Self {
        Self::new(None, None, source)
    }

    /// Discard detailed metadata and recover the original host error.
    #[must_use]
    pub fn into_source(self) -> SubnetCatalogHostError {
        self.source
    }
}

///
/// SubnetCatalogLoadFailure
///
/// Complete typed failure returned by detailed Subnet Catalog load APIs.
///

#[derive(Debug)]
pub struct SubnetCatalogLoadFailure {
    /// Requested network, selected source, and minimum assurance.
    pub request: SubnetCatalogLoadFailureRequest,
    /// Exact load stage.
    pub stage: SubnetCatalogLoadStage,
    /// Exact Registry version when known.
    pub registry_version: Option<u64>,
    /// Failure-side cache state or action.
    pub cache_disposition: SubnetCatalogFailureCacheDisposition,
    /// Typed offending identity when known.
    pub subject: Option<SubnetCatalogSubject>,
    /// Stable machine-readable source-error code.
    pub code: SubnetCatalogErrorCode,
    /// Stable operational source-error category.
    pub category: SubnetCatalogErrorCategory,
    /// Typed retry classification, including truthful unknown results.
    pub retryability: SubnetCatalogRetryability,
    /// Original host error without string conversion or replacement.
    pub source: SubnetCatalogHostError,
}

impl SubnetCatalogLoadFailure {
    pub(super) fn from_source_failure(
        request: &SubnetCatalogLoadRequest,
        stage: SubnetCatalogLoadStage,
        cache_disposition: SubnetCatalogFailureCacheDisposition,
        failure: SubnetCatalogSourceFailure,
    ) -> Self {
        let code = failure.source.code();
        let category = failure.source.category();
        let retryability = failure.source.retryability();
        Self {
            request: SubnetCatalogLoadFailureRequest::from_load_request(request),
            stage,
            registry_version: failure.registry_version,
            cache_disposition,
            subject: failure.subject,
            code,
            category,
            retryability,
            source: failure.source,
        }
    }

    /// Discard detailed provenance and recover the original host error.
    #[must_use]
    pub fn into_source(self) -> SubnetCatalogHostError {
        self.source
    }
}

impl fmt::Display for SubnetCatalogLoadFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Subnet Catalog load failed at {}: {}",
            self.stage.as_str(),
            self.source
        )
    }
}

impl Error for SubnetCatalogLoadFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

pub(super) fn subject_from_catalog_error(error: &CatalogError) -> Option<SubnetCatalogSubject> {
    match error {
        CatalogError::NetworkMismatch { actual, .. } => {
            Some(SubnetCatalogSubject::Network(actual.clone()))
        }
        CatalogError::RegistryCanisterMismatch { .. } => Some(SubnetCatalogSubject::Field(
            SubnetCatalogField::RegistryCanister,
        )),
        CatalogError::InvalidRegistryVersion => Some(SubnetCatalogSubject::Field(
            SubnetCatalogField::RegistryVersion,
        )),
        CatalogError::InvalidSourceEndpoint { endpoint, .. } => {
            Some(SubnetCatalogSubject::Endpoint(endpoint.clone()))
        }
        CatalogError::InvalidTimestamp { .. } | CatalogError::FutureTimestamp { .. } => Some(
            SubnetCatalogSubject::Field(SubnetCatalogField::CollectionTimestamp),
        ),
        CatalogError::InvalidPrincipal { field, .. } => subject_from_principal_field(field),
        CatalogError::InvalidProvenance { .. } => {
            Some(SubnetCatalogSubject::Field(SubnetCatalogField::Provenance))
        }
        CatalogError::InvalidAgreementDigest { .. }
        | CatalogError::AgreementDigestMismatch { .. } => Some(SubnetCatalogSubject::Field(
            SubnetCatalogField::AgreementDigest,
        )),
        CatalogError::InvalidCatalogDigest { .. } | CatalogError::CatalogDigestMismatch { .. } => {
            Some(SubnetCatalogSubject::Field(
                SubnetCatalogField::CatalogDigest,
            ))
        }
        CatalogError::DuplicateSubnet { subnet_principal }
        | CatalogError::UnknownRoutingSubnet { subnet_principal }
        | CatalogError::SubnetKindMismatch {
            subnet_principal, ..
        }
        | CatalogError::ChargingPolicyMismatch {
            subnet_principal, ..
        }
        | CatalogError::ClassificationMismatch {
            subnet_principal, ..
        } => {
            Principal::from_text(subnet_principal)
                .ok()
                .map(|subnet| SubnetCatalogSubject::Subnet {
                    subnet,
                    field: matches!(error, CatalogError::ClassificationMismatch { .. })
                        .then_some(SubnetCatalogField::Classification),
                })
        }
        CatalogError::InvalidRoutingRange {
            start_canister_id,
            end_canister_id,
            subnet_principal,
        } => Some(SubnetCatalogSubject::RoutingRange {
            range: RoutingRange {
                start_canister_id: start_canister_id.clone(),
                end_canister_id: end_canister_id.clone(),
                subnet_principal: subnet_principal.clone(),
            },
            field: None,
        }),
        CatalogError::OverlappingRoutingRanges { first, .. }
        | CatalogError::NonCanonicalRoutingOrder {
            previous: first, ..
        } => Some(SubnetCatalogSubject::RoutingRange {
            range: first.as_ref().clone(),
            field: None,
        }),
        CatalogError::Json(_)
        | CatalogError::UnsupportedSchemaVersion { .. }
        | CatalogError::EmptySubnets
        | CatalogError::EmptyRoutingRanges
        | CatalogError::NonCanonicalSubnetOrder { .. }
        | CatalogError::UnsupportedAssurance { .. }
        | CatalogError::ClassificationPolicyVersionMismatch { .. }
        | CatalogError::ClassificationPolicyDigestMismatch { .. }
        | CatalogError::ResolverPolicyMismatch { .. }
        | CatalogError::UnknownSubnet { .. }
        | CatalogError::PrincipalPrefixNotFound { .. }
        | CatalogError::AmbiguousPrincipalPrefix { .. }
        | CatalogError::RouteNotFound { .. } => None,
    }
}

fn subject_from_principal_field(field: &str) -> Option<SubnetCatalogSubject> {
    let field = match field {
        "provenance.registry_canister_id" => SubnetCatalogField::RegistryCanister,
        "subnet_principal" => SubnetCatalogField::SubnetPrincipal,
        "routing_range.subnet_principal" => SubnetCatalogField::RoutingTableSubnetId,
        "start_canister_id" => SubnetCatalogField::RoutingRangeStart,
        "end_canister_id" => SubnetCatalogField::RoutingRangeEnd,
        _ => return None,
    };
    Some(SubnetCatalogSubject::Field(field))
}
