//! Module: subnet_catalog::model::validation
//!
//! Responsibility: validate raw structural and host authority catalog evidence.
//! Does not own: Registry transport, cache policy, or report rendering.
//! Boundary: only this module constructs `ValidatedSubnetCatalog`.

#[cfg(feature = "subnet-catalog-host")]
use super::{
    CatalogAssurance, CatalogValidationContext, ValidatedSubnetCatalog,
    policy::{RESOLVER_BACKEND, apply_mainnet_classification_policy, classification_policy_digest},
};
use super::{RawSubnetCatalog, RoutingRange, SubnetInfo};
use crate::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, CatalogError, MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID,
    parse_principal, principal_bytes, resolver::routing_range_sorts_after,
};
#[cfg(feature = "subnet-catalog-host")]
use crate::{
    hex::{hex_bytes, is_lowercase_hex},
    http_endpoint::parse_http_endpoint,
    subnet_catalog::{CLASSIFICATION_SCHEMA_VERSION, RESOLVER_SCHEMA_VERSION},
};
#[cfg(feature = "subnet-catalog-host")]
use sha2::{Digest, Sha256};
use std::{cmp::Ordering, collections::BTreeSet};

impl RawSubnetCatalog {
    /// Build, canonicalize, classify, and seal one uncertified mainnet source snapshot.
    #[cfg(feature = "subnet-catalog-host")]
    pub fn new_mainnet_uncertified(
        registry_version: u64,
        source_endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
        collector_version: impl Into<String>,
        subnets: Vec<SubnetInfo>,
        routing_ranges: Vec<RoutingRange>,
    ) -> Result<Self, CatalogError> {
        let mut raw = Self {
            catalog_schema_version: CATALOG_SCHEMA_VERSION,
            provenance: super::SubnetCatalogProvenance {
                network: MAINNET_NETWORK.to_string(),
                registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
                registry_version,
                assurance: CatalogAssurance::UncertifiedQuery,
                source_endpoints: vec![source_endpoint.into()],
                fetched_at: fetched_at.into(),
                certificate_time: None,
                root_key_digest: None,
                fetched_by: fetched_by.into(),
                collector_version: collector_version.into(),
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
        self.subnets
            .sort_by(|left, right| left.subnet_principal.cmp(&right.subnet_principal));
        let mut keyed_ranges = self
            .routing_ranges
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
        self.routing_ranges = keyed_ranges
            .into_iter()
            .map(|(_, _, _, range)| range)
            .collect();
        apply_mainnet_classification_policy(self);
        self.provenance.classification_schema_version = CLASSIFICATION_SCHEMA_VERSION;
        self.provenance.classification_policy_digest = classification_policy_digest();
        self.provenance.resolver_schema_version = RESOLVER_SCHEMA_VERSION;
        self.provenance.resolver_backend = RESOLVER_BACKEND.to_string();
        self.catalog_digest = hex_bytes(&canonical_catalog_digest(self)?);
        self.validate()
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
        if self.subnets.is_empty() {
            return Err(CatalogError::EmptySubnets);
        }
        if self.routing_ranges.is_empty() {
            return Err(CatalogError::EmptyRoutingRanges);
        }
        parse_principal(
            &self.provenance.registry_canister_id,
            "provenance.registry_canister_id",
        )?;

        let mut subnet_principals = BTreeSet::new();
        let mut previous_subnet: Option<&str> = None;
        for subnet in &self.subnets {
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

        let mut validated_ranges = Vec::with_capacity(self.routing_ranges.len());
        for range in &self.routing_ranges {
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
impl ValidatedSubnetCatalog {
    /// Validate raw authority evidence against caller-owned identity and time policy.
    pub fn try_from_raw(
        raw: RawSubnetCatalog,
        context: &CatalogValidationContext,
    ) -> Result<Self, CatalogError> {
        raw.validate()?;
        validate_expected_identity(&raw, context)?;
        validate_provenance(&raw, context)?;
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
) -> Result<(), CatalogError> {
    let fetched_at_unix_secs = crate::subnet_catalog::parse_utc_timestamp_secs(
        &raw.provenance.fetched_at,
    )
    .ok_or_else(|| CatalogError::InvalidTimestamp {
        field: "provenance.fetched_at",
        value: raw.provenance.fetched_at.clone(),
    })?;
    if crate::subnet_catalog::format_utc_timestamp_secs(fetched_at_unix_secs)
        != raw.provenance.fetched_at
    {
        return Err(CatalogError::InvalidTimestamp {
            field: "provenance.fetched_at",
            value: raw.provenance.fetched_at.clone(),
        });
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
    if raw.provenance.source_endpoints.is_empty() {
        return Err(CatalogError::InvalidProvenance {
            field: "provenance.source_endpoints",
            reason: "at least one source endpoint is required".to_string(),
        });
    }
    for endpoint in &raw.provenance.source_endpoints {
        parse_http_endpoint(endpoint).map_err(|reason| CatalogError::InvalidSourceEndpoint {
            endpoint: endpoint.clone(),
            reason,
        })?;
    }
    match raw.provenance.assurance {
        CatalogAssurance::UncertifiedQuery => {
            if raw.provenance.source_endpoints.len() != 1 {
                return Err(CatalogError::InvalidProvenance {
                    field: "provenance.source_endpoints",
                    reason: "uncertified query assurance requires exactly one source endpoint"
                        .to_string(),
                });
            }
            if raw.provenance.certificate_time.is_some() || raw.provenance.root_key_digest.is_some()
            {
                return Err(CatalogError::InvalidProvenance {
                    field: "provenance.assurance",
                    reason: "uncertified query must not carry certificate evidence".to_string(),
                });
            }
        }
        assurance => {
            return Err(CatalogError::UnsupportedAssurance {
                assurance: assurance.as_str().to_string(),
            });
        }
    }
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
    apply_mainnet_classification_policy(&mut expected);
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
