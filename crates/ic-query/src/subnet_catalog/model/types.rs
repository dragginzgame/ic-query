//! Module: subnet_catalog::model::types
//!
//! Responsibility: define raw persisted and validated subnet catalog records.
//!
//! Does not own: validation rules, host cache paths, report shaping, or CLI filters.
//!
//! Boundary: serialized input remains untrusted until it is converted into a
//! privately held validated catalog by the host authority boundary.

use super::{ClassificationSource, GeographicScope, SubnetKind, SubnetSpecialization};
use serde::{Deserialize, Serialize};
use std::fmt;

///
/// CatalogAssurance
///
/// Authority level established for one Registry-backed catalog snapshot.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogAssurance {
    /// Registry state reconstructed from a verified certified delta sequence.
    Certified,
    /// Canonical raw Registry evidence agreed across independent endpoints.
    MultiEndpointAgreement,
    /// Version-consistent ordinary query evidence from one replica endpoint.
    UncertifiedQuery,
}

impl CatalogAssurance {
    /// Return the stable JSON and report label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Certified => "certified",
            Self::MultiEndpointAgreement => "multi_endpoint_agreement",
            Self::UncertifiedQuery => "uncertified_query",
        }
    }
}

///
/// SubnetCatalogProvenance
///
/// Registry, collection, assurance, and policy identity for one raw catalog.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogProvenance {
    /// Canonical network identity.
    pub network: String,
    /// Registry canister principal used by the collector.
    pub registry_canister_id: String,
    /// Exact Registry version shared by every joined read.
    pub registry_version: u64,
    /// Assurance actually established by the collector.
    pub assurance: CatalogAssurance,
    /// Canonically ordered source endpoints contributing to the snapshot.
    pub source_endpoints: Vec<String>,
    /// Caller-supplied UTC collection timestamp.
    pub fetched_at: String,
    /// Optional verified certificate timestamp.
    pub certificate_time: Option<String>,
    /// Optional lowercase SHA-256 digest of the trusted root key.
    pub root_key_digest: Option<String>,
    /// Collector implementation name.
    pub fetched_by: String,
    /// Collector package version.
    pub collector_version: String,
    /// Classification contract version.
    pub classification_schema_version: u32,
    /// Lowercase SHA-256 digest of the classification policy.
    pub classification_policy_digest: String,
    /// Resolver contract version.
    pub resolver_schema_version: u32,
    /// Resolver implementation identity.
    pub resolver_backend: String,
}

///
/// RawSubnetCatalog
///
/// Untrusted serialized subnet catalog representation from a source or cache.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RawSubnetCatalog {
    /// Persisted catalog schema version.
    pub catalog_schema_version: u32,
    /// Registry and collection provenance supplied with the snapshot.
    pub provenance: SubnetCatalogProvenance,
    /// Lowercase SHA-256 digest of the canonical authority payload.
    pub catalog_digest: String,
    /// Canonically ordered Subnet rows.
    pub subnets: Vec<SubnetInfo>,
    /// Canonically ordered inclusive routing ranges.
    pub routing_ranges: Vec<RoutingRange>,
}

///
/// CatalogValidationContext
///
/// Caller-owned identity and time policy for validating raw catalog evidence.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogValidationContext {
    /// Required network identity.
    pub expected_network: String,
    /// Required Registry canister principal.
    pub expected_registry_canister_id: String,
    /// Caller-supplied current Unix time.
    pub now_unix_secs: u64,
    /// Maximum accepted future timestamp skew.
    pub max_future_skew_seconds: u64,
}

impl CatalogValidationContext {
    /// Build deterministic catalog validation policy.
    #[must_use]
    pub fn new(
        expected_network: impl Into<String>,
        expected_registry_canister_id: impl Into<String>,
        now_unix_secs: u64,
        max_future_skew_seconds: u64,
    ) -> Self {
        Self {
            expected_network: expected_network.into(),
            expected_registry_canister_id: expected_registry_canister_id.into(),
            now_unix_secs,
            max_future_skew_seconds,
        }
    }
}

///
/// ValidatedSubnetCatalog
///
/// Authority-bearing catalog whose raw content passed host validation.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedSubnetCatalog {
    raw: RawSubnetCatalog,
    catalog_digest: [u8; 32],
}

impl ValidatedSubnetCatalog {
    #[cfg(feature = "subnet-catalog-host")]
    pub(in crate::subnet_catalog) const fn from_validated_parts(
        raw: RawSubnetCatalog,
        catalog_digest: [u8; 32],
    ) -> Self {
        Self {
            raw,
            catalog_digest,
        }
    }

    /// Return the immutable raw evidence that passed validation.
    #[must_use]
    pub const fn raw(&self) -> &RawSubnetCatalog {
        &self.raw
    }

    /// Return the validated provenance.
    #[must_use]
    pub const fn provenance(&self) -> &SubnetCatalogProvenance {
        &self.raw.provenance
    }

    /// Return the validated canonical Subnet rows.
    #[must_use]
    pub fn subnets(&self) -> &[SubnetInfo] {
        &self.raw.subnets
    }

    /// Return the validated canonical routing ranges.
    #[must_use]
    pub fn routing_ranges(&self) -> &[RoutingRange] {
        &self.raw.routing_ranges
    }

    /// Return the validated binary catalog digest.
    #[must_use]
    pub const fn catalog_digest(&self) -> [u8; 32] {
        self.catalog_digest
    }

    /// Return a clone of the raw catalog for serialization or evidence storage.
    #[must_use]
    pub fn to_raw(&self) -> RawSubnetCatalog {
        self.raw.clone()
    }

    /// Consume the validated wrapper and return its raw serialized evidence.
    #[must_use]
    pub fn into_raw(self) -> RawSubnetCatalog {
        self.raw
    }

    /// Find one validated Subnet row by canonical principal text.
    #[must_use]
    pub fn subnet_by_principal(&self, subnet_principal: &str) -> Option<&SubnetInfo> {
        self.raw
            .subnets
            .iter()
            .find(|subnet| subnet.subnet_principal == subnet_principal)
    }

    /// Return the validated routing ranges assigned to one Subnet.
    #[must_use]
    pub fn routing_ranges_for_subnet(&self, subnet_principal: &str) -> Vec<&RoutingRange> {
        self.raw
            .routing_ranges
            .iter()
            .filter(|range| range.subnet_principal == subnet_principal)
            .collect()
    }
}

///
/// SubnetInfo
///
/// One Subnet entry, raw Registry type code, and classification metadata.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetInfo {
    /// Canonical Subnet principal text.
    pub subnet_principal: String,
    /// Raw numeric `SubnetType` discriminant from the Registry record.
    pub registry_subnet_type: i32,
    /// IC-native classification derived from `registry_subnet_type`.
    pub subnet_kind: SubnetKind,
    /// Source of `subnet_kind`.
    pub subnet_kind_source: ClassificationSource,
    /// Curated or default Subnet specialization.
    pub subnet_specialization: SubnetSpecialization,
    /// Source of `subnet_specialization`.
    pub subnet_specialization_source: ClassificationSource,
    /// Curated or default geographic scope.
    pub geographic_scope: GeographicScope,
    /// Source of `geographic_scope`.
    pub geographic_scope_source: ClassificationSource,
    /// Human-facing policy label.
    pub subnet_label: String,
    /// Source of `subnet_label`.
    pub subnet_label_source: ClassificationSource,
    /// Node membership count reported by the Subnet record.
    pub node_count: Option<u32>,
    /// Whether application charging applies by default for the raw Subnet kind.
    pub charges_apply_by_default: bool,
}

///
/// RoutingRange
///
/// Inclusive canister routing range assigned to one Subnet.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoutingRange {
    /// Inclusive range start canister principal.
    pub start_canister_id: String,
    /// Inclusive range end canister principal.
    pub end_canister_id: String,
    /// Target Subnet principal.
    pub subnet_principal: String,
}

impl fmt::Display for RoutingRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}..{} for {}",
            self.start_canister_id, self.end_canister_id, self.subnet_principal
        )
    }
}
