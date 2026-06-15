use super::{NnsNodeProviderHostError, NnsNodeProviderListReport, NnsNodeProviderRow};
use crate::subnet_catalog::canonical_principal_text;

pub(super) fn resolve_node_provider(
    report: &NnsNodeProviderListReport,
    input: &str,
) -> Result<(NnsNodeProviderRow, String), NnsNodeProviderHostError> {
    if let Ok(principal) = canonical_principal_text(input)
        && let Some(provider) = report
            .node_providers
            .iter()
            .find(|provider| provider.node_provider_principal == principal)
    {
        return Ok((provider.clone(), "node_provider_principal".to_string()));
    }

    let prefix = input.trim().to_ascii_lowercase();
    if prefix.is_empty() {
        return Err(NnsNodeProviderHostError::NodeProviderNotFound {
            input: input.to_string(),
        });
    }
    let matches = report
        .node_providers
        .iter()
        .filter(|provider| provider.node_provider_principal.starts_with(&prefix))
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [provider] => Ok((
            provider.clone(),
            "node_provider_principal_prefix".to_string(),
        )),
        [] => Err(NnsNodeProviderHostError::NodeProviderNotFound {
            input: input.to_string(),
        }),
        _ => Err(NnsNodeProviderHostError::AmbiguousNodeProviderPrefix {
            prefix,
            matches: matches
                .into_iter()
                .map(|provider| provider.node_provider_principal)
                .collect(),
        }),
    }
}
