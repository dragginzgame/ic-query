use super::{NnsNodeOperatorHostError, NnsNodeOperatorListReport};
use crate::nns::inventory::{
    NnsInventoryInputKind, NnsInventoryResolveError, resolve_nns_inventory_row,
};

pub(super) fn resolve_node_operator(
    report: &NnsNodeOperatorListReport,
    input: &str,
) -> Result<(usize, String), NnsNodeOperatorHostError> {
    resolve_nns_inventory_row(
        &report.node_operators,
        |row| row.node_operator_principal.as_str(),
        input,
        NnsInventoryInputKind::Principal,
        "node_operator_principal",
    )
    .map_err(|error| match error {
        NnsInventoryResolveError::NotFound { input } => {
            NnsNodeOperatorHostError::NodeOperatorNotFound { input }
        }
        NnsInventoryResolveError::Ambiguous { prefix, matches } => {
            NnsNodeOperatorHostError::AmbiguousNodeOperatorPrefix { prefix, matches }
        }
    })
}
