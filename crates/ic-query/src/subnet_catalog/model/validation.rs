//! Module: subnet_catalog::model::validation
//!
//! Responsibility: validate raw structural and host authority catalog evidence.
//! Does not own: Registry transport, cache policy, or report rendering.
//! Boundary: only this module constructs `ValidatedSubnetCatalog`.

#[cfg(feature = "subnet-catalog-host")]
use super::UncertifiedCatalogCollection;
#[cfg(feature = "subnet-catalog-host")]
use super::{
    CatalogAssurance, CatalogValidationContext, ValidatedSubnetCatalog,
    policy::{RESOLVER_BACKEND, apply_mainnet_classification_policy, classification_policy_digest},
};
use super::{RawSubnetCatalog, RoutingRange, SubnetInfo};
#[cfg(feature = "certified-subnet-catalog-host")]
use crate::nns::registry::NnsAuthenticatedRegistryArchive;
use crate::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, CatalogError, MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID,
    parse_principal, principal_bytes, resolver::routing_range_sorts_after,
};
#[cfg(feature = "subnet-catalog-host")]
use crate::{
    hex::{hex_bytes, is_lowercase_hex},
    http_endpoint::parse_http_endpoint,
    subnet_catalog::{
        CLASSIFICATION_SCHEMA_VERSION, MAX_SUBNET_CATALOG_AGREEMENT_ENDPOINTS,
        MIN_SUBNET_CATALOG_AGREEMENT_ENDPOINTS, RESOLVER_SCHEMA_VERSION,
    },
};
#[cfg(feature = "subnet-catalog-host")]
use sha2::{Digest, Sha256};
use std::{cmp::Ordering, collections::BTreeSet};

impl RawSubnetCatalog {
    /// Build, canonicalize, classify, and seal one uncertified mainnet source snapshot.
    #[cfg(feature = "subnet-catalog-host")]
    pub fn new_mainnet_uncertified(
        collection: UncertifiedCatalogCollection,
        subnets: Vec<SubnetInfo>,
        routing_ranges: Vec<RoutingRange>,
    ) -> Result<Self, CatalogError> {
        let mut raw = Self {
            catalog_schema_version: CATALOG_SCHEMA_VERSION,
            provenance: super::SubnetCatalogProvenance {
                network: MAINNET_NETWORK.to_string(),
                registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
                registry_version: collection.registry_version,
                assurance: CatalogAssurance::UncertifiedQuery,
                source_endpoints: vec![collection.source_endpoint],
                agreement_digest: None,
                registry_query_call_count: collection.registry_query_call_count,
                routing_source: collection.routing_source,
                registry_records: collection.registry_records,
                fetched_at: collection.fetched_at,
                certified_registry: None,
                fetched_by: collection.fetched_by,
                collector_version: collection.collector_version,
                classification_schema_version: CLASSIFICATION_SCHEMA_VERSION,
                classification_policy_digest: classification_policy_digest(),
                resolver_schema_version: RESOLVER_SCHEMA_VERSION,
                resolver_backend: RESOLVER_BACKEND.to_string(),
            },
            catalog_digest: String::new(),
            subnets,
            routing_ranges,
        };
        raw.canonicalize_and_seal()?;
        Ok(raw)
    }

    /// Canonicalize source rows, apply the current policy, and replace the catalog digest.
    #[cfg(feature = "subnet-catalog-host")]
    pub fn canonicalize_and_seal(&mut self) -> Result<(), CatalogError> {
        canonicalize_subnet_catalog_content(&mut self.subnets, &mut self.routing_ranges)?;
        self.provenance.registry_records.sort_by(|left, right| {
            left.source_endpoint
                .cmp(&right.source_endpoint)
                .then_with(|| left.record.key.cmp(&right.record.key))
                .then_with(|| left.record.kind.as_str().cmp(right.record.kind.as_str()))
                .then_with(|| {
                    left.returned_registry_version
                        .cmp(&right.returned_registry_version)
                })
        });
        self.provenance.classification_schema_version = CLASSIFICATION_SCHEMA_VERSION;
        self.provenance.classification_policy_digest = classification_policy_digest();
        self.provenance.resolver_schema_version = RESOLVER_SCHEMA_VERSION;
        self.provenance.resolver_backend = RESOLVER_BACKEND.to_string();
        self.catalog_digest = hex_bytes(&canonical_catalog_digest(self)?);
        self.validate()
    }

    /// Promote matching single-endpoint evidence into one sealed agreement snapshot.
    #[cfg(feature = "subnet-catalog-host")]
    pub(in crate::subnet_catalog) fn promote_to_multi_endpoint_agreement(
        &mut self,
        source_endpoints: Vec<String>,
        registry_query_call_count: u64,
    ) -> Result<(), CatalogError> {
        self.provenance.assurance = CatalogAssurance::MultiEndpointAgreement;
        self.provenance.source_endpoints = source_endpoints;
        self.provenance.registry_query_call_count = registry_query_call_count;
        self.provenance.agreement_digest = Some(hex_bytes(&catalog_agreement_digest(self)?));
        self.canonicalize_and_seal()
    }

    /// Validate schema, fixed mainnet identity, raw classifications, and routing structure.
    pub fn validate(&self) -> Result<(), CatalogError> {
        if self.catalog_schema_version != CATALOG_SCHEMA_VERSION {
            return Err(CatalogError::UnsupportedSchemaVersion {
                found: self.catalog_schema_version,
                supported: CATALOG_SCHEMA_VERSION,
            });
        }
        if self.provenance.network != MAINNET_NETWORK {
            return Err(CatalogError::NetworkMismatch {
                expected: MAINNET_NETWORK.to_string(),
                actual: self.provenance.network.clone(),
            });
        }
        if self.provenance.registry_canister_id != MAINNET_REGISTRY_CANISTER_ID {
            return Err(CatalogError::RegistryCanisterMismatch {
                expected: MAINNET_REGISTRY_CANISTER_ID.to_string(),
                actual: self.provenance.registry_canister_id.clone(),
            });
        }
        if self.provenance.registry_version == 0 {
            return Err(CatalogError::InvalidRegistryVersion);
        }
        parse_principal(
            &self.provenance.registry_canister_id,
            "provenance.registry_canister_id",
        )?;

        validate_subnet_catalog_content(&self.subnets, &self.routing_ranges)
    }

    /// Find one raw Subnet row by canonical principal text.
    #[must_use]
    pub fn subnet_by_principal(&self, subnet_principal: &str) -> Option<&SubnetInfo> {
        self.subnets
            .iter()
            .find(|subnet| subnet.subnet_principal == subnet_principal)
    }

    /// Return raw routing ranges assigned to one Subnet.
    #[must_use]
    pub fn routing_ranges_for_subnet(&self, subnet_principal: &str) -> Vec<&RoutingRange> {
        self.routing_ranges
            .iter()
            .filter(|range| range.subnet_principal == subnet_principal)
            .collect()
    }
}

#[cfg(feature = "subnet-catalog-host")]
pub fn canonicalize_subnet_catalog_content(
    subnets: &mut [SubnetInfo],
    routing_ranges: &mut Vec<RoutingRange>,
) -> Result<(), CatalogError> {
    subnets.sort_by(|left, right| left.subnet_principal.cmp(&right.subnet_principal));
    let mut keyed_ranges = routing_ranges
        .drain(..)
        .map(|range| {
            Ok::<_, CatalogError>((
                principal_bytes(&range.start_canister_id, "start_canister_id")?,
                principal_bytes(&range.end_canister_id, "end_canister_id")?,
                range.subnet_principal.clone(),
                range,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    keyed_ranges.sort_by(|left, right| {
        compare_routing_keys(&left.0, &left.1, &left.2, &right.0, &right.1, &right.2)
    });
    *routing_ranges = keyed_ranges
        .into_iter()
        .map(|(_, _, _, range)| range)
        .collect();
    apply_mainnet_classification_policy(subnets);
    validate_subnet_catalog_content(subnets, routing_ranges)
}

fn validate_subnet_catalog_content(
    subnets: &[SubnetInfo],
    routing_ranges: &[RoutingRange],
) -> Result<(), CatalogError> {
    if subnets.is_empty() {
        return Err(CatalogError::EmptySubnets);
    }
    if routing_ranges.is_empty() {
        return Err(CatalogError::EmptyRoutingRanges);
    }
    let mut subnet_principals = BTreeSet::new();
    let mut previous_subnet: Option<&str> = None;
    for subnet in subnets {
        parse_principal(&subnet.subnet_principal, "subnet_principal")?;
        if let Some(previous) = previous_subnet
            && previous >= subnet.subnet_principal.as_str()
        {
            return Err(CatalogError::NonCanonicalSubnetOrder {
                previous: previous.to_string(),
                current: subnet.subnet_principal.clone(),
            });
        }
        previous_subnet = Some(subnet.subnet_principal.as_str());
        if !subnet_principals.insert(subnet.subnet_principal.clone()) {
            return Err(CatalogError::DuplicateSubnet {
                subnet_principal: subnet.subnet_principal.clone(),
            });
        }
        validate_raw_subnet_classification(subnet)?;
    }

    let mut validated_ranges = Vec::with_capacity(routing_ranges.len());
    for range in routing_ranges {
        if !subnet_principals.contains(&range.subnet_principal) {
            return Err(CatalogError::UnknownRoutingSubnet {
                subnet_principal: range.subnet_principal.clone(),
            });
        }
        let start = principal_bytes(&range.start_canister_id, "start_canister_id")?;
        let end = principal_bytes(&range.end_canister_id, "end_canister_id")?;
        parse_principal(&range.subnet_principal, "routing_range.subnet_principal")?;
        if routing_range_sorts_after(&start, &end) {
            return Err(CatalogError::InvalidRoutingRange {
                subnet_principal: range.subnet_principal.clone(),
                start_canister_id: range.start_canister_id.clone(),
                end_canister_id: range.end_canister_id.clone(),
            });
        }
        validated_ranges.push((range, start, end));
    }
    for pair in validated_ranges.windows(2) {
        let (first, first_start, first_end) = &pair[0];
        let (second, second_start, second_end) = &pair[1];
        if compare_routing_keys(
            first_start,
            first_end,
            &first.subnet_principal,
            second_start,
            second_end,
            &second.subnet_principal,
        ) != Ordering::Less
        {
            return Err(CatalogError::NonCanonicalRoutingOrder {
                previous: Box::new((*first).clone()),
                current: Box::new((*second).clone()),
            });
        }
        if second_start <= first_end {
            return Err(CatalogError::OverlappingRoutingRanges {
                first: Box::new((*first).clone()),
                second: Box::new((*second).clone()),
            });
        }
    }

    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
impl ValidatedSubnetCatalog {
    /// Validate raw authority evidence against caller-owned identity and time policy.
    pub fn try_from_raw(
        raw: RawSubnetCatalog,
        context: &CatalogValidationContext,
    ) -> Result<Self, CatalogError> {
        Self::try_from_raw_with_certified_admission(raw, context, false)
    }

    #[cfg(feature = "certified-subnet-catalog-host")]
    pub(crate) fn try_from_authenticated_archive(
        raw: RawSubnetCatalog,
        context: &CatalogValidationContext,
        archive: &NnsAuthenticatedRegistryArchive,
    ) -> Result<Self, CatalogError> {
        validate_certified_archive_binding(&raw, archive)?;
        Self::try_from_raw_with_certified_admission(raw, context, true)
    }

    fn try_from_raw_with_certified_admission(
        raw: RawSubnetCatalog,
        context: &CatalogValidationContext,
        admit_certified: bool,
    ) -> Result<Self, CatalogError> {
        raw.validate()?;
        validate_expected_identity(&raw, context)?;
        validate_provenance(&raw, context, admit_certified)?;
        validate_classification_policy(&raw)?;
        let catalog_digest = validate_catalog_digest(&raw)?;
        Ok(Self::from_validated_parts(raw, catalog_digest))
    }
}

fn validate_raw_subnet_classification(subnet: &SubnetInfo) -> Result<(), CatalogError> {
    let expected_kind = super::SubnetKind::from_registry_subnet_type(subnet.registry_subnet_type);
    if subnet.subnet_kind != expected_kind {
        return Err(CatalogError::SubnetKindMismatch {
            subnet_principal: subnet.subnet_principal.clone(),
            registry_subnet_type: subnet.registry_subnet_type,
            expected: expected_kind.as_str().to_string(),
            actual: subnet.subnet_kind.as_str().to_string(),
        });
    }
    if subnet.subnet_kind_source != super::ClassificationSource::Registry {
        return Err(CatalogError::ClassificationMismatch {
            subnet_principal: subnet.subnet_principal.clone(),
            field: "subnet_kind_source",
            reason: "raw Registry subnet kind must have registry source".to_string(),
        });
    }
    let expected_charges = expected_kind.charges_apply_by_default();
    if subnet.charges_apply_by_default != expected_charges {
        return Err(CatalogError::ChargingPolicyMismatch {
            subnet_principal: subnet.subnet_principal.clone(),
            expected: expected_charges,
            actual: subnet.charges_apply_by_default,
        });
    }
    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_expected_identity(
    raw: &RawSubnetCatalog,
    context: &CatalogValidationContext,
) -> Result<(), CatalogError> {
    if raw.provenance.network != context.expected_network {
        return Err(CatalogError::NetworkMismatch {
            expected: context.expected_network.clone(),
            actual: raw.provenance.network.clone(),
        });
    }
    if raw.provenance.registry_canister_id != context.expected_registry_canister_id {
        return Err(CatalogError::RegistryCanisterMismatch {
            expected: context.expected_registry_canister_id.clone(),
            actual: raw.provenance.registry_canister_id.clone(),
        });
    }
    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_provenance(
    raw: &RawSubnetCatalog,
    context: &CatalogValidationContext,
    admit_certified: bool,
) -> Result<(), CatalogError> {
    validate_collection_time(raw, context)?;
    validate_collector_identity(raw)?;
    let parsed_endpoints = validated_source_endpoints(raw)?;
    validate_assurance(raw, &parsed_endpoints, admit_certified)?;
    validate_registry_record_evidence(raw)?;
    validate_policy_identity(raw)
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_registry_record_evidence(raw: &RawSubnetCatalog) -> Result<(), CatalogError> {
    if raw.provenance.registry_records.is_empty() {
        return Ok(());
    }

    let expected_subnets = raw
        .subnets
        .iter()
        .map(|subnet| subnet.subnet_principal.clone())
        .collect::<BTreeSet<_>>();
    let mut subnet_list_endpoints = BTreeSet::new();
    let mut routing_endpoints = BTreeSet::new();
    let mut subnet_record_identities = BTreeSet::new();
    let mut previous_record = None;
    for evidence in &raw.provenance.registry_records {
        if evidence.requested_registry_version != raw.provenance.registry_version {
            return Err(invalid_registry_record_evidence(
                "record request version does not match the pinned catalog version".to_string(),
            ));
        }
        if evidence.returned_registry_version > evidence.requested_registry_version {
            return Err(invalid_registry_record_evidence(
                "record value version is newer than its requested version".to_string(),
            ));
        }
        if !raw
            .provenance
            .source_endpoints
            .contains(&evidence.source_endpoint)
        {
            return Err(invalid_registry_record_evidence(
                "record source endpoint is not a catalog source endpoint".to_string(),
            ));
        }
        if evidence.assurance != CatalogAssurance::UncertifiedQuery {
            return Err(invalid_registry_record_evidence(
                "individual get_value evidence must have uncertified-query assurance".to_string(),
            ));
        }

        let record_identity = (
            evidence.source_endpoint.as_str(),
            evidence.record.key.as_str(),
        );
        if previous_record.is_some_and(|previous| previous >= record_identity) {
            return Err(invalid_registry_record_evidence(
                "record evidence is duplicated or not in canonical endpoint/key order".to_string(),
            ));
        }
        previous_record = Some(record_identity);

        match validated_registry_record_subject(
            evidence,
            raw.provenance.routing_source,
            &expected_subnets,
        )? {
            RegistryRecordEvidenceSubject::SubnetList => {
                subnet_list_endpoints.insert(evidence.source_endpoint.as_str());
            }
            RegistryRecordEvidenceSubject::Routing => {
                routing_endpoints.insert(evidence.source_endpoint.as_str());
            }
            RegistryRecordEvidenceSubject::Subnet(subnet) => {
                subnet_record_identities.insert((evidence.source_endpoint.as_str(), subnet));
            }
        }
    }

    for endpoint in &raw.provenance.source_endpoints {
        if !subnet_list_endpoints.contains(endpoint.as_str()) {
            return Err(invalid_registry_record_evidence(format!(
                "source endpoint {endpoint:?} has no Subnet-list record evidence"
            )));
        }
        if !routing_endpoints.contains(endpoint.as_str()) {
            return Err(invalid_registry_record_evidence(format!(
                "source endpoint {endpoint:?} has no selected routing record evidence"
            )));
        }
        for subnet in &expected_subnets {
            if !subnet_record_identities.contains(&(endpoint.as_str(), subnet.clone())) {
                return Err(invalid_registry_record_evidence(format!(
                    "source endpoint {endpoint:?} has no record evidence for Subnet {subnet}"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
enum RegistryRecordEvidenceSubject {
    SubnetList,
    Routing,
    Subnet(String),
}

#[cfg(feature = "subnet-catalog-host")]
fn validated_registry_record_subject(
    evidence: &super::SubnetCatalogRegistryRecordEvidence,
    routing_source: super::SubnetCatalogRoutingSource,
    expected_subnets: &BTreeSet<String>,
) -> Result<RegistryRecordEvidenceSubject, CatalogError> {
    match evidence.record.kind {
        super::SubnetCatalogRegistryRecordKind::SubnetList => {
            if evidence.record.key != crate::ic_registry::SUBNET_LIST_KEY
                || evidence.record.subnet.is_some()
                || evidence.record.canister_range_start.is_some()
            {
                return Err(invalid_registry_record_evidence(
                    "Subnet-list evidence does not match its exact Registry subject".to_string(),
                ));
            }
            Ok(RegistryRecordEvidenceSubject::SubnetList)
        }
        super::SubnetCatalogRegistryRecordKind::RoutingTable => {
            if evidence.record.subnet.is_some() {
                return Err(invalid_registry_record_evidence(
                    "routing evidence unexpectedly names a Subnet-record subject".to_string(),
                ));
            }
            let is_modern = if evidence.record.key == crate::ic_registry::ROUTING_TABLE_KEY {
                if evidence.record.canister_range_start.is_some() {
                    return Err(invalid_registry_record_evidence(
                        "legacy routing evidence has a shard lower-bound subject".to_string(),
                    ));
                }
                false
            } else {
                let parsed = crate::ic_registry::routing_shards::canister_range_start_from_key(
                    crate::ic_registry::CANISTER_RANGES_KEY_PREFIX,
                    &evidence.record.key,
                )
                .map_err(invalid_registry_record_evidence)?;
                if evidence.record.canister_range_start.as_ref() != Some(&parsed) {
                    return Err(invalid_registry_record_evidence(
                        "routing shard subject does not match the lower bound encoded by its key"
                            .to_string(),
                    ));
                }
                true
            };
            let selected_modern =
                routing_source == super::SubnetCatalogRoutingSource::CanisterRanges;
            if is_modern != selected_modern {
                return Err(invalid_registry_record_evidence(
                    "routing record evidence contradicts the selected routing source".to_string(),
                ));
            }
            Ok(RegistryRecordEvidenceSubject::Routing)
        }
        super::SubnetCatalogRegistryRecordKind::SubnetRecord => {
            let Some(subnet) = evidence.record.subnet else {
                return Err(invalid_registry_record_evidence(
                    "Subnet-record evidence has no Subnet subject".to_string(),
                ));
            };
            let subnet = subnet.to_text();
            if evidence.record.canister_range_start.is_some()
                || evidence.record.key != crate::ic_registry::subnet_record_key(&subnet)
                || !expected_subnets.contains(&subnet)
            {
                return Err(invalid_registry_record_evidence(
                    "Subnet-record evidence does not match a current catalog Subnet".to_string(),
                ));
            }
            Ok(RegistryRecordEvidenceSubject::Subnet(subnet))
        }
    }
}

#[cfg(feature = "subnet-catalog-host")]
const fn invalid_registry_record_evidence(reason: String) -> CatalogError {
    CatalogError::InvalidProvenance {
        field: "provenance.registry_records",
        reason,
    }
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_collection_time(
    raw: &RawSubnetCatalog,
    context: &CatalogValidationContext,
) -> Result<(), CatalogError> {
    let invalid_timestamp = || CatalogError::InvalidTimestamp {
        field: "provenance.fetched_at",
        value: raw.provenance.fetched_at.clone(),
    };
    let fetched_at_unix_secs =
        crate::subnet_catalog::parse_utc_timestamp_secs(&raw.provenance.fetched_at)
            .ok_or_else(invalid_timestamp)?;
    if crate::subnet_catalog::format_utc_timestamp_secs(fetched_at_unix_secs)
        != raw.provenance.fetched_at
    {
        return Err(invalid_timestamp());
    }
    let latest_allowed = context
        .now_unix_secs
        .saturating_add(context.max_future_skew_seconds);
    if fetched_at_unix_secs > latest_allowed {
        return Err(CatalogError::FutureTimestamp {
            field: "provenance.fetched_at",
            value: raw.provenance.fetched_at.clone(),
            latest_allowed_unix_secs: latest_allowed,
        });
    }
    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_collector_identity(raw: &RawSubnetCatalog) -> Result<(), CatalogError> {
    if raw.provenance.fetched_by.trim().is_empty() {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.fetched_by",
            reason: "collector identity must not be empty".to_string(),
        });
    }
    if raw.provenance.collector_version.trim().is_empty() {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.collector_version",
            reason: "collector version must not be empty".to_string(),
        });
    }
    if raw.provenance.registry_query_call_count == 0 {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.registry_query_call_count",
            reason: "catalog evidence must record at least one Registry query call".to_string(),
        });
    }
    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
fn validated_source_endpoints(raw: &RawSubnetCatalog) -> Result<Vec<url::Url>, CatalogError> {
    if raw.provenance.source_endpoints.is_empty() {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.source_endpoints",
            reason: "at least one source endpoint is required".to_string(),
        });
    }
    raw.provenance
        .source_endpoints
        .iter()
        .map(|endpoint| {
            parse_http_endpoint(endpoint).map_err(|reason| CatalogError::InvalidSourceEndpoint {
                endpoint: endpoint.clone(),
                reason,
            })
        })
        .collect()
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_assurance(
    raw: &RawSubnetCatalog,
    parsed_endpoints: &[url::Url],
    admit_certified: bool,
) -> Result<(), CatalogError> {
    match raw.provenance.assurance {
        CatalogAssurance::UncertifiedQuery => {
            if raw.provenance.source_endpoints.len() != 1 {
                return Err(CatalogError::InvalidProvenance {
                    field: "provenance.source_endpoints",
                    reason: "uncertified query assurance requires exactly one source endpoint"
                        .to_string(),
                });
            }
            if raw.provenance.certified_registry.is_some() {
                return Err(CatalogError::InvalidProvenance {
                    field: "provenance.assurance",
                    reason: "uncertified query must not carry certificate evidence".to_string(),
                });
            }
            if raw.provenance.agreement_digest.is_some() {
                return Err(CatalogError::InvalidProvenance {
                    field: "provenance.agreement_digest",
                    reason: "uncertified query must not claim endpoint agreement".to_string(),
                });
            }
        }
        CatalogAssurance::MultiEndpointAgreement => {
            let endpoint_count = raw.provenance.source_endpoints.len();
            if !(MIN_SUBNET_CATALOG_AGREEMENT_ENDPOINTS..=MAX_SUBNET_CATALOG_AGREEMENT_ENDPOINTS)
                .contains(&endpoint_count)
            {
                return Err(CatalogError::InvalidProvenance {
                    field: "provenance.source_endpoints",
                    reason: format!(
                        "multi-endpoint agreement requires {MIN_SUBNET_CATALOG_AGREEMENT_ENDPOINTS}..={MAX_SUBNET_CATALOG_AGREEMENT_ENDPOINTS} endpoints"
                    ),
                });
            }
            if raw
                .provenance
                .source_endpoints
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            {
                return Err(CatalogError::InvalidProvenance {
                    field: "provenance.source_endpoints",
                    reason: "agreement endpoints must be unique and canonically ordered"
                        .to_string(),
                });
            }
            let mut hostnames = BTreeSet::new();
            for endpoint in parsed_endpoints {
                let hostname =
                    endpoint
                        .host_str()
                        .ok_or_else(|| CatalogError::InvalidProvenance {
                            field: "provenance.source_endpoints",
                            reason: "agreement endpoint is missing a hostname".to_string(),
                        })?;
                if !hostnames.insert(hostname.to_ascii_lowercase()) {
                    return Err(CatalogError::InvalidProvenance {
                        field: "provenance.source_endpoints",
                        reason: "agreement endpoints must use distinct hostnames".to_string(),
                    });
                }
            }
            if raw.provenance.certified_registry.is_some() {
                return Err(CatalogError::InvalidProvenance {
                    field: "provenance.assurance",
                    reason: "multi-endpoint agreement must not carry certificate evidence"
                        .to_string(),
                });
            }
            validate_agreement_digest(raw)?;
        }
        CatalogAssurance::Certified => {
            if !admit_certified {
                return Err(CatalogError::UnsupportedAssurance {
                    assurance: CatalogAssurance::Certified.as_str().to_string(),
                });
            }
            validate_certified_assurance(raw)?;
        }
    }
    Ok(())
}

#[cfg(feature = "certified-subnet-catalog-host")]
fn validate_certified_archive_binding(
    raw: &RawSubnetCatalog,
    archive: &NnsAuthenticatedRegistryArchive,
) -> Result<(), CatalogError> {
    let manifest = archive.manifest();
    let expected_evidence = super::CertifiedRegistryCatalogEvidence {
        archive_manifest_schema_version: manifest.schema_version,
        delta_report_schema_version: manifest.delta_report_schema_version,
        replay_provenance_schema_version: manifest.replay_provenance_schema_version,
        root_key_digest: manifest.root_key_digest.clone(),
        evidence_chain_digest: manifest.evidence_chain_digest.clone(),
        complete_state_digest: manifest.complete_state_digest.clone(),
        minimum_certificate_time_nanos: manifest.minimum_certificate_time_nanos,
        maximum_certificate_time_nanos: manifest.maximum_certificate_time_nanos,
    };
    if raw.provenance.assurance != CatalogAssurance::Certified
        || raw.provenance.network != manifest.network
        || raw.provenance.registry_canister_id != manifest.registry_canister_id
        || raw.provenance.registry_version != manifest.selected_version
        || raw.provenance.source_endpoints != manifest.source_endpoints
        || raw.provenance.registry_query_call_count != manifest.query_call_count
        || raw.provenance.certified_registry.as_ref() != Some(&expected_evidence)
    {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance",
            reason: "certified catalog provenance does not match its authenticated archive"
                .to_string(),
        });
    }
    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_certified_assurance(raw: &RawSubnetCatalog) -> Result<(), CatalogError> {
    if raw.provenance.agreement_digest.is_some() {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.agreement_digest",
            reason: "certified assurance must not claim endpoint agreement".to_string(),
        });
    }
    if raw
        .provenance
        .source_endpoints
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
    {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.source_endpoints",
            reason: "certified archive endpoints must be unique and canonically ordered"
                .to_string(),
        });
    }
    let evidence = raw.provenance.certified_registry.as_ref().ok_or_else(|| {
        CatalogError::InvalidProvenance {
            field: "provenance.certified_registry",
            reason: "certified assurance requires authenticated archive commitments".to_string(),
        }
    })?;
    for (field, digest) in [
        ("root_key_digest", &evidence.root_key_digest),
        ("evidence_chain_digest", &evidence.evidence_chain_digest),
        ("complete_state_digest", &evidence.complete_state_digest),
    ] {
        if digest.len() != 64 || !is_lowercase_hex(digest) {
            return Err(CatalogError::InvalidProvenance {
                field: "provenance.certified_registry",
                reason: format!("{field} must be exactly 32 lowercase hexadecimal bytes"),
            });
        }
    }
    if evidence.archive_manifest_schema_version == 0
        || evidence.delta_report_schema_version == 0
        || evidence.replay_provenance_schema_version == 0
    {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.certified_registry",
            reason: "certified evidence schema versions must be greater than zero".to_string(),
        });
    }
    if evidence.minimum_certificate_time_nanos > evidence.maximum_certificate_time_nanos {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.certified_registry",
            reason: "minimum certificate time exceeds maximum certificate time".to_string(),
        });
    }
    let maximum_certificate_time = crate::subnet_catalog::format_utc_timestamp_secs(
        evidence.maximum_certificate_time_nanos / 1_000_000_000,
    );
    if raw.provenance.fetched_at != maximum_certificate_time {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.fetched_at",
            reason: "certified catalog time must equal the latest archive certificate time"
                .to_string(),
        });
    }
    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_policy_identity(raw: &RawSubnetCatalog) -> Result<(), CatalogError> {
    if raw.provenance.classification_schema_version != CLASSIFICATION_SCHEMA_VERSION {
        return Err(CatalogError::ClassificationPolicyVersionMismatch {
            found: raw.provenance.classification_schema_version,
            supported: CLASSIFICATION_SCHEMA_VERSION,
        });
    }
    let expected_policy_digest = classification_policy_digest();
    if raw.provenance.classification_policy_digest != expected_policy_digest {
        return Err(CatalogError::ClassificationPolicyDigestMismatch {
            expected: expected_policy_digest,
            actual: raw.provenance.classification_policy_digest.clone(),
        });
    }
    if raw.provenance.resolver_schema_version != RESOLVER_SCHEMA_VERSION
        || raw.provenance.resolver_backend != RESOLVER_BACKEND
    {
        return Err(CatalogError::ResolverPolicyMismatch {
            expected_version: RESOLVER_SCHEMA_VERSION,
            actual_version: raw.provenance.resolver_schema_version,
            expected_backend: RESOLVER_BACKEND.to_string(),
            actual_backend: raw.provenance.resolver_backend.clone(),
        });
    }
    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_classification_policy(raw: &RawSubnetCatalog) -> Result<(), CatalogError> {
    let mut expected = raw.clone();
    apply_mainnet_classification_policy(&mut expected.subnets);
    for (actual, expected) in raw.subnets.iter().zip(&expected.subnets) {
        if actual.subnet_specialization != expected.subnet_specialization
            || actual.subnet_specialization_source != expected.subnet_specialization_source
            || actual.geographic_scope != expected.geographic_scope
            || actual.geographic_scope_source != expected.geographic_scope_source
            || actual.subnet_label != expected.subnet_label
            || actual.subnet_label_source != expected.subnet_label_source
        {
            return Err(CatalogError::ClassificationMismatch {
                subnet_principal: actual.subnet_principal.clone(),
                field: "curated_or_computed_annotations",
                reason: "annotations do not match the recorded classification policy".to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_catalog_digest(raw: &RawSubnetCatalog) -> Result<[u8; 32], CatalogError> {
    if raw.catalog_digest.len() != 64 || !is_lowercase_hex(&raw.catalog_digest) {
        return Err(CatalogError::InvalidCatalogDigest {
            value: raw.catalog_digest.clone(),
        });
    }
    let expected = canonical_catalog_digest(raw)?;
    if raw.catalog_digest != hex_bytes(&expected) {
        return Err(CatalogError::CatalogDigestMismatch {
            expected: hex_bytes(&expected),
            actual: raw.catalog_digest.clone(),
        });
    }
    Ok(expected)
}

#[cfg(feature = "subnet-catalog-host")]
fn canonical_catalog_digest(raw: &RawSubnetCatalog) -> Result<[u8; 32], CatalogError> {
    let mut payload = raw.clone();
    payload.catalog_digest.clear();
    let serialized = serde_json::to_vec(&payload)?;
    Ok(Sha256::digest(serialized).into())
}

#[cfg(feature = "subnet-catalog-host")]
pub(in crate::subnet_catalog) fn catalog_agreement_digest(
    raw: &RawSubnetCatalog,
) -> Result<[u8; 32], CatalogError> {
    #[derive(serde::Serialize)]
    struct AgreementPayload<'a> {
        catalog_schema_version: u32,
        network: &'a str,
        registry_canister_id: &'a str,
        registry_version: u64,
        routing_source: super::SubnetCatalogRoutingSource,
        registry_records: Vec<AgreementRegistryRecord<'a>>,
        subnets: &'a [SubnetInfo],
        routing_ranges: &'a [RoutingRange],
    }

    #[derive(serde::Serialize)]
    struct AgreementRegistryRecord<'a> {
        record: &'a super::SubnetCatalogRegistryRecordSubject,
        requested_registry_version: u64,
        returned_registry_version: u64,
        timestamp_nanoseconds: u64,
        value_encoding: super::SubnetCatalogRegistryValueEncoding,
    }

    let registry_records = raw
        .provenance
        .registry_records
        .iter()
        .map(|evidence| AgreementRegistryRecord {
            record: &evidence.record,
            requested_registry_version: evidence.requested_registry_version,
            returned_registry_version: evidence.returned_registry_version,
            timestamp_nanoseconds: evidence.timestamp_nanoseconds,
            value_encoding: evidence.value_encoding,
        })
        .collect();

    let payload = AgreementPayload {
        catalog_schema_version: raw.catalog_schema_version,
        network: &raw.provenance.network,
        registry_canister_id: &raw.provenance.registry_canister_id,
        registry_version: raw.provenance.registry_version,
        routing_source: raw.provenance.routing_source,
        registry_records,
        subnets: &raw.subnets,
        routing_ranges: &raw.routing_ranges,
    };
    Ok(Sha256::digest(serde_json::to_vec(&payload)?).into())
}

#[cfg(feature = "subnet-catalog-host")]
fn validate_agreement_digest(raw: &RawSubnetCatalog) -> Result<(), CatalogError> {
    let actual = raw
        .provenance
        .agreement_digest
        .as_deref()
        .unwrap_or_default();
    if actual.len() != 64 || !is_lowercase_hex(actual) {
        return Err(CatalogError::InvalidAgreementDigest {
            value: actual.to_string(),
        });
    }
    let expected = hex_bytes(&catalog_agreement_digest(raw)?);
    if actual != expected {
        return Err(CatalogError::AgreementDigestMismatch {
            expected,
            actual: actual.to_string(),
        });
    }
    Ok(())
}

fn compare_routing_keys(
    left_start: &[u8],
    left_end: &[u8],
    left_subnet: &str,
    right_start: &[u8],
    right_end: &[u8],
    right_subnet: &str,
) -> Ordering {
    left_start
        .cmp(right_start)
        .then_with(|| left_end.cmp(right_end))
        .then_with(|| left_subnet.cmp(right_subnet))
}
