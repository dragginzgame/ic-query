use super::{ResolveAs, ResolvedSubnet};
use crate::subnet_catalog::{CatalogError, RawSubnetCatalog, parse_principal};
use std::collections::BTreeSet;

impl RawSubnetCatalog {
    /// Resolve an exact principal or a unique cached subnet principal prefix.
    pub fn resolve_principal_or_prefix(
        &self,
        input: &str,
        forced: Option<ResolveAs>,
    ) -> Result<ResolvedSubnet, CatalogError> {
        if let Ok(principal) = parse_principal(input, "principal") {
            return self.resolve_parsed_principal(&principal, forced);
        }
        self.resolve_principal_prefix(input, forced)
    }

    /// Resolve a unique prefix of a cached subnet principal.
    pub fn resolve_principal_prefix(
        &self,
        input: &str,
        forced: Option<ResolveAs>,
    ) -> Result<ResolvedSubnet, CatalogError> {
        let prefix = input.trim().to_ascii_lowercase();
        if prefix.is_empty() {
            return Err(CatalogError::PrincipalPrefixNotFound { prefix });
        }

        let matches = self.subnet_principal_prefix_matches(&prefix, forced);
        let mut iter = matches.iter();
        let Some(first) = iter.next() else {
            return Err(CatalogError::PrincipalPrefixNotFound { prefix });
        };
        if iter.next().is_some() {
            return Err(CatalogError::AmbiguousPrincipalPrefix {
                prefix,
                matches: matches
                    .iter()
                    .map(|subnet| format!("subnet:{subnet}"))
                    .collect::<Vec<_>>(),
            });
        }

        let mut resolved = self.resolve_known_subnet(first)?;
        resolved.input_principal = input.to_string();
        resolved.resolved_from = "subnet_principal_prefix".to_string();
        Ok(resolved)
    }

    fn subnet_principal_prefix_matches(
        &self,
        prefix: &str,
        forced: Option<ResolveAs>,
    ) -> BTreeSet<&str> {
        let mut matches = BTreeSet::new();
        if forced != Some(ResolveAs::Canister) {
            for subnet in &self.subnets {
                if subnet.subnet_principal.starts_with(prefix) {
                    matches.insert(subnet.subnet_principal.as_str());
                }
            }
        }
        matches
    }
}
