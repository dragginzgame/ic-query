use super::{NnsNodeProviderHostError, NnsNodeProviderListReport, NnsNodeProviderRow};
use crate::nns::inventory::{
    NnsInventoryInputKind, NnsInventoryResolveError, NnsInventoryRow, resolve_nns_inventory_row,
};

impl NnsInventoryRow for NnsNodeProviderRow {
    fn inventory_id(&self) -> &str {
        &self.node_provider_principal
    }
}

pub(super) fn resolve_node_provider(
    report: &NnsNodeProviderListReport,
    input: &str,
) -> Result<(NnsNodeProviderRow, String), NnsNodeProviderHostError> {
    resolve_nns_inventory_row(
        &report.node_providers,
        input,
        NnsInventoryInputKind::Principal,
        "node_provider_principal",
        "node_provider_principal_prefix",
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
