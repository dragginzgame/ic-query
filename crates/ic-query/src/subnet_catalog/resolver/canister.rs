use super::{ResolvedCanisterRoute, ResolvedSubnet, ResolvedSubnetSubject};
use crate::subnet_catalog::{
    CatalogError, RawSubnetCatalog, RoutingRange, SubnetInfo, ValidatedSubnetCatalog,
    parse_principal, principal_bytes,
};
use ic_platform::candid::contract::Principal;

impl RawSubnetCatalog {
    /// Resolve a canister principal through cached routing ranges.
    pub fn resolve_canister(&self, input_principal: &str) -> Result<ResolvedSubnet, CatalogError> {
        let canister = parse_principal(input_principal, "canister_principal")?;
        let canonical_canister = canister.to_text();
        let (range, subnet) = self.canister_route(&canister)?;
        Ok(ResolvedSubnet {
            input_principal: canonical_canister.clone(),
            resolved_as: ResolvedSubnetSubject::Canister,
            resolved_from: "routing_range".to_string(),
            subnet: subnet.clone(),
            matched_canister_principal: Some(canonical_canister),
            matched_routing_range: Some(range.clone()),
            catalog_digest: self.catalog_digest.clone(),
            provenance: self.provenance.clone(),
        })
    }

    fn canister_route(
        &self,
        canister: &Principal,
    ) -> Result<(&RoutingRange, &SubnetInfo), CatalogError> {
        let range = self
            .routing_ranges
            .iter()
            .find(|range| range_contains_principal(range, canister.as_slice()).unwrap_or(false))
            .ok_or_else(|| CatalogError::RouteNotFound {
                canister_principal: canister.to_text(),
                registry_version: self.provenance.registry_version,
                catalog_schema_version: self.catalog_schema_version,
            })?;
        let subnet = self
            .subnet_by_principal(&range.subnet_principal)
            .ok_or_else(|| CatalogError::UnknownRoutingSubnet {
                subnet_principal: range.subnet_principal.clone(),
            })?;
        Ok((range, subnet))
    }
}

impl ValidatedSubnetCatalog {
    /// Resolve one canister to authority evidence bound to this exact catalog.
    pub fn resolve_canister_route(
        &self,
        input_principal: &str,
    ) -> Result<ResolvedCanisterRoute, CatalogError> {
        let canister = parse_principal(input_principal, "canister_principal")?;
        let (matched_range, subnet) = self.raw().canister_route(&canister)?;
        Ok(ResolvedCanisterRoute {
            canister,
            subnet: parse_principal(&matched_range.subnet_principal, "subnet_principal")?,
            subnet_info: subnet.clone(),
            matched_range: matched_range.clone(),
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
