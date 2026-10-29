use super::{NnsNodeProviderHostError, NnsNodeProviderListReport};
use crate::nns::inventory::{
    NnsInventoryInputKind, NnsInventoryResolveError, resolve_nns_inventory_row,
};

pub(super) fn resolve_node_provider(
    report: &NnsNodeProviderListReport,
    input: &str,
) -> Result<(usize, String), NnsNodeProviderHostError> {
    resolve_nns_inventory_row(
        &report.node_providers,
        |row| row.node_provider_principal.as_str(),
        input,
        NnsInventoryInputKind::Principal,
        "node_provider_principal",
    )
    .map_err(|error| match error {
        NnsInventoryResolveError::NotFound { input } => {
            NnsNodeProviderHostError::NodeProviderNotFound { input }
        }
        NnsInventoryResolveError::Ambiguous { prefix, matches } => {
            NnsNodeProviderHostError::AmbiguousNodeProviderPrefix { prefix, matches }
        }
    })
}
