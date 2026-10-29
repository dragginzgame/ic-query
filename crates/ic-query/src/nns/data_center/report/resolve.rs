use super::{NnsDataCenterHostError, NnsDataCenterListReport};
use crate::nns::inventory::{
    NnsInventoryInputKind, NnsInventoryResolveError, resolve_nns_inventory_row,
};

pub(super) fn resolve_data_center(
    report: &NnsDataCenterListReport,
    input: &str,
) -> Result<(usize, String), NnsDataCenterHostError> {
    resolve_nns_inventory_row(
        &report.data_centers,
        |row| row.data_center_id.as_str(),
        input,
        NnsInventoryInputKind::Text,
        "data_center_id",
    )
    .map_err(|error| match error {
        NnsInventoryResolveError::NotFound { input } => {
            NnsDataCenterHostError::DataCenterNotFound { input }
        }
        NnsInventoryResolveError::Ambiguous { prefix, matches } => {
            NnsDataCenterHostError::AmbiguousDataCenterPrefix { prefix, matches }
        }
    })
}
