use super::{NnsDataCenterHostError, NnsDataCenterListReport, NnsDataCenterRow};
use crate::nns::inventory::{
    NnsInventoryInputKind, NnsInventoryResolveError, NnsInventoryRow, resolve_nns_inventory_row,
};

impl NnsInventoryRow for NnsDataCenterRow {
    fn inventory_id(&self) -> &str {
        &self.data_center_id
    }
}

pub(super) fn resolve_data_center(
    report: &NnsDataCenterListReport,
    input: &str,
) -> Result<(NnsDataCenterRow, String), NnsDataCenterHostError> {
    resolve_nns_inventory_row(
        &report.data_centers,
        input,
        NnsInventoryInputKind::Text,
        "data_center_id",
        "data_center_id_prefix",
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
