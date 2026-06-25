use super::{ResolveAs, ResolvedSubnet, ResolvedSubnetSubject};
use crate::subnet_catalog::{CatalogError, SubnetCatalog, canonical_principal_text};

impl SubnetCatalog {
    /// Resolve a principal as a known subnet or as a canister covered by a cached range.
    pub fn resolve_principal(
        &self,
        input: &str,
        forced: Option<ResolveAs>,
    ) -> Result<ResolvedSubnet, CatalogError> {
        let input_principal = canonical_principal_text(input)?;
        match forced {
            Some(ResolveAs::Subnet) => self.resolve_known_subnet(&input_principal),
            None if self.subnet_by_principal(&input_principal).is_some() => {
                self.resolve_known_subnet(&input_principal)
            }
            Some(ResolveAs::Canister) | None => self.resolve_canister(&input_principal),
        }
    }

    pub(super) fn resolve_known_subnet(
        &self,
        input_principal: &str,
    ) -> Result<ResolvedSubnet, CatalogError> {
        let subnet = self
            .subnet_by_principal(input_principal)
            .cloned()
            .ok_or_else(|| CatalogError::UnknownSubnet {
                subnet_principal: input_principal.to_string(),
            })?;
        Ok(ResolvedSubnet {
            input_principal: input_principal.to_string(),
            resolved_as: ResolvedSubnetSubject::Subnet,
            resolved_from: "subnet_principal".to_string(),
            subnet,
            matched_canister_principal: None,
            matched_routing_range: None,
        })
    }
}
