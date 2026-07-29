mod data_center;
mod node;
mod node_operator;
mod node_provider;
mod subnet_topology;

use crate::{ic_registry::proto::SubnetType, subnet_catalog::SubnetKind};

pub(super) use data_center::data_center_list_from_inventory;
pub(super) use node::node_list_from_inventory;
pub(super) use node_operator::node_operator_list_from_inventory;
#[cfg(test)]
pub(super) use node_provider::node_provider_from_governance;
pub(super) use node_provider::node_provider_list_from_response;
pub(super) use subnet_topology::subnet_topology_from_inventory;

pub(in crate::ic_registry) fn subnet_kind_from_registry(subnet_type: i32) -> SubnetKind {
    match SubnetType::try_from(subnet_type).ok() {
        Some(SubnetType::Application | SubnetType::VerifiedApplication) => SubnetKind::Application,
        Some(SubnetType::CloudEngine) => SubnetKind::CloudEngine,
        Some(SubnetType::System) => SubnetKind::System,
        Some(SubnetType::Unspecified) | None => SubnetKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_subnet_type_conversion_preserves_raw_ic_query_kinds() {
        let cases = [
            (SubnetType::Application as i32, SubnetKind::Application),
            (
                SubnetType::VerifiedApplication as i32,
                SubnetKind::Application,
            ),
            (SubnetType::System as i32, SubnetKind::System),
            (SubnetType::CloudEngine as i32, SubnetKind::CloudEngine),
            (SubnetType::Unspecified as i32, SubnetKind::Unknown),
            (i32::MAX, SubnetKind::Unknown),
        ];

        for (raw_subnet_type, expected) in cases {
            assert_eq!(subnet_kind_from_registry(raw_subnet_type), expected);
        }
    }
}
