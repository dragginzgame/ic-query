#[cfg(feature = "host")]
mod data_center;
#[cfg(feature = "host")]
mod node;
#[cfg(feature = "host")]
mod node_operator;
#[cfg(feature = "host")]
mod node_provider;
#[cfg(feature = "host")]
mod subnet_topology;

#[cfg(all(test, feature = "host"))]
use crate::ic_registry::proto::SubnetType;
use crate::subnet_catalog::SubnetKind;

#[cfg(feature = "host")]
pub(super) use data_center::data_center_list_from_inventory;
#[cfg(feature = "host")]
pub(super) use node::node_list_from_inventory;
#[cfg(feature = "host")]
pub(super) use node_operator::node_operator_list_from_inventory;
#[cfg(all(test, feature = "host"))]
pub(super) use node_provider::node_provider_from_governance;
#[cfg(feature = "host")]
pub(super) use node_provider::node_provider_list_from_response;
#[cfg(feature = "host")]
pub(super) use subnet_topology::subnet_topology_from_inventory;

pub(in crate::ic_registry) const fn subnet_kind_from_registry(subnet_type: i32) -> SubnetKind {
    SubnetKind::from_registry_subnet_type(subnet_type)
}

#[cfg(all(test, feature = "host"))]
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
