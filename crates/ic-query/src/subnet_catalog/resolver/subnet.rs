use super::{ResolveAs, ResolvedSubnet, ResolvedSubnetSubject};
use crate::subnet_catalog::{CatalogError, RawSubnetCatalog, SubnetInfo, parse_principal};
use ic_platform::candid::contract::Principal;

impl RawSubnetCatalog {
    /// Resolve a principal as a known subnet or as a canister covered by a cached range.
    pub fn resolve_principal(
        &self,
        input: &str,
        forced: Option<ResolveAs>,
    ) -> Result<ResolvedSubnet, CatalogError> {
        let principal = parse_principal(input, "principal")?;
        self.resolve_parsed_principal(&principal, forced)
    }

    pub(super) fn resolve_parsed_principal(
        &self,
        principal: &Principal,
        forced: Option<ResolveAs>,
    ) -> Result<ResolvedSubnet, CatalogError> {
        let input_principal = principal.to_text();
        match forced {
            Some(ResolveAs::Subnet) => self.resolve_known_subnet(&input_principal),
            None => self.subnet_by_principal(&input_principal).map_or_else(
                || self.resolve_parsed_canister(principal),
                |subnet| Ok(self.resolved_subnet(&input_principal, subnet)),
            ),
            Some(ResolveAs::Canister) => self.resolve_parsed_canister(principal),
        }
    }

    pub(super) fn resolve_known_subnet(
        &self,
        input_principal: &str,
    ) -> Result<ResolvedSubnet, CatalogError> {
        let subnet = self.subnet_by_principal(input_principal).ok_or_else(|| {
            CatalogError::UnknownSubnet {
                subnet_principal: input_principal.to_string(),
            }
        })?;
        Ok(self.resolved_subnet(input_principal, subnet))
    }

    pub(super) fn resolved_subnet(
        &self,
        input_principal: &str,
        subnet: &SubnetInfo,
    ) -> ResolvedSubnet {
        ResolvedSubnet {
            input_principal: input_principal.to_string(),
            resolved_as: ResolvedSubnetSubject::Subnet,
            resolved_from: "subnet_principal".to_string(),
            subnet: subnet.clone(),
            matched_canister_principal: None,
            matched_routing_range: None,
            catalog_digest: self.catalog_digest.clone(),
            provenance: self.provenance.clone(),
        }
    }
}
