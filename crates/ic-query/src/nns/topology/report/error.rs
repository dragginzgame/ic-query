use crate::{nns::NnsInventoryError, subnet_catalog::SubnetCatalogHostError};
use thiserror::Error as ThisError;

///
/// NnsTopologyHostError
///
/// Errors returned while reading or refreshing an NNS topology report.
///

#[derive(Debug, ThisError)]
pub enum NnsTopologyHostError {
    #[error("NNS topology reports support only the mainnet `ic` network (requested {network:?})")]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Subnet(#[from] SubnetCatalogHostError),

    /// One Registry inventory failed, retaining its typed collection and source.
    #[error(transparent)]
    Inventory(#[from] NnsInventoryError),
}
