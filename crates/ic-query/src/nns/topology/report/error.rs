use crate::{
    nns::{
        data_center::report::NnsDataCenterHostError, node::report::NnsNodeHostError,
        node_operator::report::NnsNodeOperatorHostError,
        node_provider::report::NnsNodeProviderHostError,
    },
    subnet_catalog::SubnetCatalogHostError,
};
use thiserror::Error as ThisError;

///
/// NnsTopologyHostError
///
#[derive(Debug, ThisError)]
pub enum NnsTopologyHostError {
    #[error(
        "`icq nns topology` supports only the mainnet `ic` network\n\nThe NNS topology report is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not implemented yet.\n\nTry:\n  icq --network ic nns topology summary\n  icq --network ic nns topology coverage\n  icq --network ic nns topology versions\n  icq --network ic nns topology health\n  icq --network ic nns topology gaps\n  icq --network ic nns topology capacity\n  icq --network ic nns topology regions\n  icq --network ic nns topology providers\n  icq --network ic nns topology refresh"
    )]
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
