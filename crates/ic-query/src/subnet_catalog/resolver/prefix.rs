use super::{ResolveAs, ResolvedSubnet};
use crate::identifier_match::{IdentifierMatch, match_identifier};
use crate::subnet_catalog::{CatalogError, RawSubnetCatalog, parse_principal};
use std::collections::BTreeMap;

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

        let mut identifiers = BTreeMap::new();
        if forced != Some(ResolveAs::Canister) {
            for subnet in &self.subnets {
                identifiers
                    .entry(subnet.subnet_principal.as_str())
                    .or_insert(subnet);
            }
        }
        let matched = match match_identifier(
            identifiers.into_values(),
            |subnet| subnet.subnet_principal.as_str(),
            None,
            &prefix,
        ) {
            IdentifierMatch::Prefix(subnet) => subnet,
            IdentifierMatch::Missing => {
                return Err(CatalogError::PrincipalPrefixNotFound { prefix });
            }
            IdentifierMatch::Ambiguous(matches) => {
                return Err(CatalogError::AmbiguousPrincipalPrefix {
                    prefix,
                    matches: matches
                        .into_iter()
                        .map(|subnet| format!("subnet:{}", subnet.subnet_principal))
                        .collect(),
                });
            }
            IdentifierMatch::Exact(_) => {
                unreachable!("prefix lookup supplies no exact identifier")
            }
        };
        let mut resolved = self.resolved_subnet(&matched.subnet_principal, matched);
        resolved.input_principal = input.to_string();
        resolved.resolved_from = "subnet_principal_prefix".to_string();
        Ok(resolved)
    }
}
