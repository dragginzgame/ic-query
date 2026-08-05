use super::{ResolvedCanisterRoute, ResolvedSubnet, ResolvedSubnetSubject};
use crate::subnet_catalog::{
    CatalogError, RawSubnetCatalog, ValidatedSubnetCatalog, parse_principal, principal_bytes,
};

impl RawSubnetCatalog {
    /// Resolve a canister principal through cached routing ranges.
    pub fn resolve_canister(&self, input_principal: &str) -> Result<ResolvedSubnet, CatalogError> {
        let canonical_canister = parse_principal(input_principal, "canister_principal")?.to_text();
        let canister_bytes = principal_bytes(&canonical_canister, "canister_principal")?;
        let range = self
            .routing_ranges
            .iter()
            .find(|range| range_contains_principal(range, &canister_bytes).unwrap_or(false))
            .ok_or_else(|| CatalogError::RouteNotFound {
                canister_principal: canonical_canister.clone(),
                registry_version: self.provenance.registry_version,
                catalog_schema_version: self.catalog_schema_version,
            })?;
        let subnet = self
            .subnet_by_principal(&range.subnet_principal)
            .cloned()
            .ok_or_else(|| CatalogError::UnknownRoutingSubnet {
                subnet_principal: range.subnet_principal.clone(),
            })?;
        Ok(ResolvedSubnet {
            input_principal: canonical_canister.clone(),
            resolved_as: ResolvedSubnetSubject::Canister,
            resolved_from: "routing_range".to_string(),
            subnet,
            matched_canister_principal: Some(canonical_canister),
            matched_routing_range: Some(range.clone()),
            catalog_digest: self.catalog_digest.clone(),
            provenance: self.provenance.clone(),
        })
    }
}

impl ValidatedSubnetCatalog {
    /// Resolve one canister to authority evidence bound to this exact catalog.
    pub fn resolve_canister_route(
        &self,
        input_principal: &str,
    ) -> Result<ResolvedCanisterRoute, CatalogError> {
        let resolved = self.raw().resolve_canister(input_principal)?;
        let canister_text = resolved
            .matched_canister_principal
            .as_deref()
            .ok_or_else(|| CatalogError::RouteNotFound {
                canister_principal: input_principal.to_string(),
                registry_version: self.provenance().registry_version,
                catalog_schema_version: self.raw().catalog_schema_version,
            })?;
        let matched_range =
            resolved
                .matched_routing_range
                .ok_or_else(|| CatalogError::RouteNotFound {
                    canister_principal: input_principal.to_string(),
                    registry_version: self.provenance().registry_version,
                    catalog_schema_version: self.raw().catalog_schema_version,
                })?;
        Ok(ResolvedCanisterRoute {
            canister: parse_principal(canister_text, "canister_principal")?,
            subnet: parse_principal(&matched_range.subnet_principal, "subnet_principal")?,
            matched_range,
            registry_version: self.provenance().registry_version,
            catalog_digest: self.catalog_digest(),
            provenance: self.provenance().clone(),
        })
    }
}

pub(in crate::subnet_catalog) fn routing_range_sorts_after(start: &[u8], end: &[u8]) -> bool {
    start > end
}

fn range_contains_principal(
    range: &crate::subnet_catalog::RoutingRange,
    principal: &[u8],
) -> Result<bool, CatalogError> {
    let start = principal_bytes(&range.start_canister_id, "start_canister_id")?;
    let end = principal_bytes(&range.end_canister_id, "end_canister_id")?;
    Ok(start.as_slice() <= principal && principal <= end.as_slice())
}
