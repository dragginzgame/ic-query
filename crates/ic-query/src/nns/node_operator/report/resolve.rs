use super::{NnsNodeOperatorHostError, NnsNodeOperatorListReport, NnsNodeOperatorRow};
use crate::nns::inventory::{
    NnsInventoryInputKind, NnsInventoryResolveError, NnsInventoryRow, resolve_nns_inventory_row,
};

impl NnsInventoryRow for NnsNodeOperatorRow {
    fn inventory_id(&self) -> &str {
        &self.node_operator_principal
    }
}

pub(super) fn resolve_node_operator(
    report: &NnsNodeOperatorListReport,
    input: &str,
) -> Result<(NnsNodeOperatorRow, String), NnsNodeOperatorHostError> {
    resolve_nns_inventory_row(
        &report.node_operators,
        input,
        NnsInventoryInputKind::Principal,
        "node_operator_principal",
        "node_operator_principal_prefix",
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
