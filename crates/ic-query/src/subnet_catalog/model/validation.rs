use super::{RoutingRange, SubnetCatalog, SubnetInfo};
use crate::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, CatalogError, parse_principal, principal_bytes,
    resolver::routing_range_sorts_after,
};
use std::collections::BTreeSet;

impl SubnetCatalog {
    /// Validate schema, principal syntax, and routing references.
    pub fn validate(&self) -> Result<(), CatalogError> {
        if self.catalog_schema_version != CATALOG_SCHEMA_VERSION {
            return Err(CatalogError::UnsupportedSchemaVersion {
                found: self.catalog_schema_version,
                supported: CATALOG_SCHEMA_VERSION,
            });
        }
        if self.subnets.is_empty() {
            return Err(CatalogError::EmptySubnets);
        }
        if self.routing_ranges.is_empty() {
            return Err(CatalogError::EmptyRoutingRanges);
        }
        parse_principal(&self.registry_canister_id, "registry_canister_id")?;

        let mut subnet_principals = BTreeSet::new();
        for subnet in &self.subnets {
            parse_principal(&subnet.subnet_principal, "subnet_principal")?;
            if !subnet_principals.insert(subnet.subnet_principal.clone()) {
                return Err(CatalogError::DuplicateSubnet {
                    subnet_principal: subnet.subnet_principal.clone(),
                });
            }
        }

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
        }

        Ok(())
    }

    #[must_use]
    pub fn subnet_by_principal(&self, subnet_principal: &str) -> Option<&SubnetInfo> {
        self.subnets
            .iter()
            .find(|subnet| subnet.subnet_principal == subnet_principal)
    }

    #[must_use]
    pub fn routing_ranges_for_subnet(&self, subnet_principal: &str) -> Vec<&RoutingRange> {
        self.routing_ranges
            .iter()
            .filter(|range| range.subnet_principal == subnet_principal)
            .collect()
    }
}
