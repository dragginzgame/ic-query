use super::{NnsNodeHostError, NnsNodeListReport};
use crate::nns::inventory::{
    NnsInventoryInputKind, NnsInventoryResolveError, resolve_nns_inventory_row,
};

pub(super) fn resolve_node(
    report: &NnsNodeListReport,
    input: &str,
) -> Result<(usize, String), NnsNodeHostError> {
    resolve_nns_inventory_row(
        &report.nodes,
        |row| row.node_principal.as_str(),
        input,
        NnsInventoryInputKind::Principal,
        "node_principal",
    )
    .map_err(|error| match error {
        NnsInventoryResolveError::NotFound { input } => NnsNodeHostError::NodeNotFound { input },
        NnsInventoryResolveError::Ambiguous { prefix, matches } => {
            NnsNodeHostError::AmbiguousNodePrefix { prefix, matches }
        }
    })
}
