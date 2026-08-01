use super::{NnsNodeHostError, NnsNodeListReport, NnsNodeRow};
use crate::nns::inventory::{
    NnsInventoryInputKind, NnsInventoryResolveError, NnsInventoryRow, resolve_nns_inventory_row,
};

impl NnsInventoryRow for NnsNodeRow {
    fn inventory_id(&self) -> &str {
        &self.node_principal
    }
}

pub(super) fn resolve_node(
    report: &NnsNodeListReport,
    input: &str,
) -> Result<(NnsNodeRow, String), NnsNodeHostError> {
    resolve_nns_inventory_row(
        &report.nodes,
        input,
        NnsInventoryInputKind::Principal,
        "node_principal",
        "node_principal_prefix",
    )
    .map_err(|error| match error {
        NnsInventoryResolveError::NotFound { input } => NnsNodeHostError::NodeNotFound { input },
        NnsInventoryResolveError::Ambiguous { prefix, matches } => {
            NnsNodeHostError::AmbiguousNodePrefix { prefix, matches }
        }
    })
}
