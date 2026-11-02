use crate::{
    nns::{
        data_center::NnsDataCenterHostError, node::NnsNodeHostError,
        node_operator::NnsNodeOperatorHostError, node_provider::NnsNodeProviderHostError,
    },
    subnet_catalog::SubnetCatalogHostError,
};
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

    #[error(transparent)]
    Node(#[from] NnsNodeHostError),

    #[error(transparent)]
    NodeProvider(#[from] NnsNodeProviderHostError),

    #[error(transparent)]
    NodeOperator(#[from] NnsNodeOperatorHostError),

    #[error(transparent)]
    DataCenter(#[from] NnsDataCenterHostError),
}
