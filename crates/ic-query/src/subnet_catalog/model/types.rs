//! Module: subnet_catalog::model::types
//!
//! Responsibility: define raw persisted and validated subnet catalog records.
//!
//! Does not own: validation rules, host cache paths, report shaping, or CLI filters.
//!
//! Boundary: serialized input remains untrusted until it is converted into a
//! privately held validated catalog by the host authority boundary.

use super::{ClassificationSource, GeographicScope, SubnetKind, SubnetSpecialization};
use candid::Principal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Exact Registry key containing the current Subnet list.
pub const SUBNET_LIST_KEY: &str = "subnet_list";
/// Retired monolithic Registry routing-table key used only by explicit historical replay.
pub const ROUTING_TABLE_KEY: &str = "routing_table";
/// Registry key prefix for authoritative canister-range routing shards.
pub const CANISTER_RANGES_KEY_PREFIX: &str = "canister_ranges_";
/// Registry key prefix for individual Subnet records.
pub const SUBNET_RECORD_KEY_PREFIX: &str = "subnet_record_";

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

///
/// SubnetCatalogRoutingSource
///
/// Registry record family selected as routing authority for one catalog.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubnetCatalogRoutingSource {
    /// The complete current `canister_ranges_*` key family.
    CanisterRanges,
    /// The retired monolithic `routing_table`, used only when no shards exist.
    LegacyRoutingTable,
}

impl SubnetCatalogRoutingSource {
    /// Return the stable JSON and report label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CanisterRanges => "canister_ranges",
            Self::LegacyRoutingTable => "legacy_routing_table",
        }
    }
}

///
/// SubnetCatalogRegistryRecordKind
///
/// Exact Registry key family and protobuf schema used by a catalog record.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubnetCatalogRegistryRecordKind {
    /// The `subnet_list` key containing a `SubnetListRecord`.
    SubnetList,
    /// A legacy `routing_table` or modern `canister_ranges_*` `RoutingTable` value.
    RoutingTable,
    /// One `subnet_record_*` key containing a `SubnetRecord`.
    SubnetRecord,
}

impl SubnetCatalogRegistryRecordKind {
    /// Return the stable source-family label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubnetList => "subnet_list",
            Self::RoutingTable => "routing_table",
            Self::SubnetRecord => "subnet_record",
        }
    }

    /// Return the exact protobuf schema decoded for this record family.
    #[must_use]
    pub const fn protobuf_schema(self) -> &'static str {
        match self {
            Self::SubnetList => "SubnetListRecord",
            Self::RoutingTable => "RoutingTable",
            Self::SubnetRecord => "SubnetRecord",
        }
    }
}

///
/// SubnetCatalogRegistryRecordSubject
///
/// Typed Registry key and domain subject retained for one catalog record.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubnetCatalogRegistryRecordSubject {
    /// Registry record family and protobuf schema.
    pub kind: SubnetCatalogRegistryRecordKind,
    /// Exact Registry key used by `get_value`.
    pub key: String,
    /// Exact Subnet principal for a Subnet-record operation.
    pub subnet: Option<Principal>,
    /// Range-start canister principal encoded in a `canister_ranges_*` key.
    pub canister_range_start: Option<Principal>,
}

impl SubnetCatalogRegistryRecordSubject {
    /// Build the exact current Subnet-list subject.
    #[must_use]
    pub fn subnet_list() -> Self {
        Self::keyed(SubnetCatalogRegistryRecordKind::SubnetList, SUBNET_LIST_KEY)
    }

    /// Build the exact retired monolithic routing-table subject.
    #[must_use]
    pub fn legacy_routing_table() -> Self {
        Self::keyed(
            SubnetCatalogRegistryRecordKind::RoutingTable,
            ROUTING_TABLE_KEY,
        )
    }

    /// Build the exact Registry subject for one Subnet record.
    #[must_use]
    pub fn subnet_record(subnet: Principal) -> Self {
        Self {
            kind: SubnetCatalogRegistryRecordKind::SubnetRecord,
            key: format!("{SUBNET_RECORD_KEY_PREFIX}{}", subnet.to_text()),
            subnet: Some(subnet),
            canister_range_start: None,
        }
    }

    /// Build the exact authoritative routing-shard subject for one lower bound.
    #[must_use]
    pub fn canister_ranges(canister_range_start: Principal) -> Self {
        Self {
            kind: SubnetCatalogRegistryRecordKind::RoutingTable,
            key: format!(
                "{CANISTER_RANGES_KEY_PREFIX}{}",
                crate::hex::hex_bytes(canister_range_start.as_slice())
            ),
            subnet: None,
            canister_range_start: Some(canister_range_start),
        }
    }

    #[must_use]
    fn keyed(kind: SubnetCatalogRegistryRecordKind, key: impl Into<String>) -> Self {
        Self {
            kind,
            key: key.into(),
            subnet: None,
            canister_range_start: None,
        }
    }

    #[cfg(feature = "subnet-catalog-host")]
    #[must_use]
    pub(crate) fn exact_keyed(
        kind: SubnetCatalogRegistryRecordKind,
        key: impl Into<String>,
    ) -> Self {
        Self::keyed(kind, key)
    }
}

///
/// SubnetCatalogRegistryValueEncoding
///
/// Registry transport representation used to complete one fetched value.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubnetCatalogRegistryValueEncoding {
    /// The complete protobuf value was returned inline.
    Inline,
    /// The protobuf value was reconstructed from hash-verified chunks.
    Chunked,
}

impl SubnetCatalogRegistryValueEncoding {
    /// Return the stable JSON and report label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Chunked => "chunked",
        }
    }
}

///
/// SubnetCatalogRegistryRecordEvidence
///
/// Exact request, returned value, source, and transport provenance for one record.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubnetCatalogRegistryRecordEvidence {
    /// Exact Registry key, family, schema, and domain subject.
    pub record: SubnetCatalogRegistryRecordSubject,
    /// Pinned Registry version requested from `get_value`.
    pub requested_registry_version: u64,
    /// Individual value version returned by the Registry.
    pub returned_registry_version: u64,
    /// Registry-assigned timestamp of the returned value's last mutation.
    pub timestamp_nanoseconds: u64,
    /// Exact endpoint that returned the value.
    pub source_endpoint: String,
    /// Assurance of the individual read, before any endpoint aggregation.
    pub assurance: CatalogAssurance,
    /// Inline or hash-verified chunked value representation.
    pub value_encoding: SubnetCatalogRegistryValueEncoding,
}

impl SubnetCatalogRegistryRecordEvidence {
    /// Build evidence for one ordinary pinned Registry value response.
    #[must_use]
    pub fn uncertified_query(
        record: SubnetCatalogRegistryRecordSubject,
        requested_registry_version: u64,
        returned_registry_version: u64,
        timestamp_nanoseconds: u64,
        source_endpoint: impl Into<String>,
        value_encoding: SubnetCatalogRegistryValueEncoding,
    ) -> Self {
        Self {
            record,
            requested_registry_version,
            returned_registry_version,
            timestamp_nanoseconds,
            source_endpoint: source_endpoint.into(),
            assurance: CatalogAssurance::UncertifiedQuery,
            value_encoding,
        }
    }
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

    /// Return whether this assurance meets the caller's required minimum.
    #[must_use]
    pub const fn satisfies(self, minimum: Self) -> bool {
        self.strength() >= minimum.strength()
    }

    const fn strength(self) -> u8 {
        match self {
            Self::UncertifiedQuery => 0,
            Self::MultiEndpointAgreement => 1,
            Self::Certified => 2,
        }
    }
}

///
/// UncertifiedCatalogCollection
///
/// Single-endpoint collection metadata used to construct one raw catalog.
///

#[cfg(feature = "subnet-catalog-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UncertifiedCatalogCollection {
    /// Exact Registry version shared by every joined read.
    pub registry_version: u64,
    /// Exact source endpoint queried by the collector.
    pub source_endpoint: String,
    /// Canonical UTC collection or latest certified-evidence timestamp.
    pub fetched_at: String,
    /// Collector implementation name.
    pub fetched_by: String,
    /// Collector package version.
    pub collector_version: String,
    /// Exact number of Registry query calls made during collection.
    pub registry_query_call_count: u64,
    /// Registry record family selected as routing authority.
    pub routing_source: SubnetCatalogRoutingSource,
    /// Canonical evidence for every fetched Registry value.
    pub registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
}

#[cfg(feature = "subnet-catalog-host")]
impl UncertifiedCatalogCollection {
    /// Build explicit single-endpoint collection metadata.
    #[must_use]
    pub fn new(
        registry_version: u64,
        source_endpoint: &str,
        fetched_at: &str,
        fetched_by: &str,
        collector_version: &str,
        registry_query_call_count: u64,
    ) -> Self {
        Self {
            registry_version,
            source_endpoint: source_endpoint.to_string(),
            fetched_at: fetched_at.to_string(),
            fetched_by: fetched_by.to_string(),
            collector_version: collector_version.to_string(),
            registry_query_call_count,
            routing_source: SubnetCatalogRoutingSource::LegacyRoutingTable,
            registry_records: Vec::new(),
        }
    }

    /// Attach explicit routing authority and per-value evidence.
    #[must_use]
    pub fn with_registry_evidence(
        mut self,
        routing_source: SubnetCatalogRoutingSource,
        registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
    ) -> Self {
        self.routing_source = routing_source;
        self.registry_records = registry_records;
        self
    }
}

///
/// CertifiedRegistryCatalogEvidence
///
/// Persistable commitments copied from one fully reauthenticated Registry archive.
/// Serialized values describe evidence but cannot establish certified authority by themselves.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CertifiedRegistryCatalogEvidence {
    /// Archive-manifest schema authenticated during archive restoration.
    pub archive_manifest_schema_version: u32,
    /// Certified Registry delta-report schema authenticated during restoration.
    pub delta_report_schema_version: u32,
    /// Replay-provenance schema governing the retained commitments.
    pub replay_provenance_schema_version: u32,
    /// Lowercase SHA-256 digest of the trusted mainnet root key.
    pub root_key_digest: String,
    /// Lowercase SHA-256 commitment to the ordered authenticated report sequence.
    pub evidence_chain_digest: String,
    /// Lowercase SHA-256 commitment to the exact reconstructed Registry state.
    pub complete_state_digest: String,
    /// Earliest authenticated certificate time in nanoseconds.
    pub minimum_certificate_time_nanos: u64,
    /// Latest authenticated certificate time in nanoseconds.
    pub maximum_certificate_time_nanos: u64,
}

///
/// SubnetCatalogProvenance
///
/// Registry, collection, assurance, and policy identity for one raw catalog.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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
    /// Canonical Registry payload digest agreed by every source endpoint.
    pub agreement_digest: Option<String>,
    /// Exact number of Registry query calls used to collect this snapshot.
    pub registry_query_call_count: u64,
    /// Registry record family selected as routing authority.
    pub routing_source: SubnetCatalogRoutingSource,
    /// Canonical evidence for every fetched Registry value.
    pub registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
    /// Caller-supplied UTC collection timestamp.
    pub fetched_at: String,
    /// Certified archive commitments, present only for certified assurance.
    pub certified_registry: Option<CertifiedRegistryCatalogEvidence>,
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
/// CatalogSnapshotAuthorityEvidence
///
/// Stable persistable identity derived only from one validated catalog snapshot.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogSnapshotAuthorityEvidence {
    /// Exact Registry version represented by the validated catalog.
    pub registry_version: u64,
    /// Lowercase SHA-256 digest of the validated catalog authority payload.
    pub catalog_digest: String,
    /// Assurance established for the validated catalog evidence.
    pub assurance: CatalogAssurance,
    /// Canonically ordered endpoints contributing to the validated evidence.
    pub source_endpoints: Vec<String>,
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

    /// Return stable persistable authority derived only from this validated snapshot.
    #[must_use]
    pub fn snapshot_authority(&self) -> CatalogSnapshotAuthorityEvidence {
        let provenance = self.provenance();
        CatalogSnapshotAuthorityEvidence {
            registry_version: provenance.registry_version,
            catalog_digest: self.raw.catalog_digest.clone(),
            assurance: provenance.assurance,
            source_endpoints: provenance.source_endpoints.clone(),
        }
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
