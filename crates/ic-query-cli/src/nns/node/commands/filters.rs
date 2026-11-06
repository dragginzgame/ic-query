use clap::builder::{PossibleValuesParser, TypedValueParser};
use ic_query::subnet_catalog::SubnetKind;

pub(in crate::nns::node) const SUBNET_FILTER_ARG: &str = "subnet";
pub(in crate::nns::node) const SUBNET_KIND_FILTER_ARG: &str = "kind";
pub(in crate::nns::node) const DATA_CENTER_FILTER_ARG: &str = "data-center";
pub(in crate::nns::node) const NODE_PROVIDER_FILTER_ARG: &str = "node-provider";
pub(in crate::nns::node) const NODE_OPERATOR_FILTER_ARG: &str = "node-operator";

///
/// NodeFilters
///
/// Typed inventory filters shared by command construction and request projection.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::nns::node) struct NodeFilters {
    #[arg(id = SUBNET_FILTER_ARG, long = SUBNET_FILTER_ARG, value_name = "subnet|subnet-prefix", help = "Show only nodes assigned to a subnet principal or prefix")]
    pub(in crate::nns::node) subnet: Option<String>,
    #[arg(id = SUBNET_KIND_FILTER_ARG, long = SUBNET_KIND_FILTER_ARG, value_name = "application|cloud_engine|system|unknown", help = "Show only nodes assigned to subnets of this kind", value_parser = PossibleValuesParser::new([ SubnetKind::Application.as_str(), SubnetKind::CloudEngine.as_str(), SubnetKind::System.as_str(), SubnetKind::Unknown.as_str(), ]) .try_map(|value| value.parse::<SubnetKind>()), )]
    pub(in crate::nns::node) subnet_kind: Option<SubnetKind>,
    #[arg(id = DATA_CENTER_FILTER_ARG, long = DATA_CENTER_FILTER_ARG, value_name = "data-center|data-center-prefix", help = "Show only nodes in a data center id or prefix")]
    pub(in crate::nns::node) data_center: Option<String>,
    #[arg(id = NODE_PROVIDER_FILTER_ARG, long = NODE_PROVIDER_FILTER_ARG, value_name = "node-provider|node-provider-prefix", help = "Show only nodes owned by a node-provider principal or prefix")]
    pub(in crate::nns::node) node_provider: Option<String>,
    #[arg(id = NODE_OPERATOR_FILTER_ARG, long = NODE_OPERATOR_FILTER_ARG, value_name = "node-operator|node-operator-prefix", help = "Show only nodes owned by a node-operator principal or prefix")]
    pub(in crate::nns::node) node_operator: Option<String>,
}
