use crate::subnet_catalog::{ResolvedSubnetSubject, SubnetInfo, SubnetKind};

const BASE_13_NODE_CYCLES_PER_BILLION_INSTRUCTIONS: u128 = 1_000_000_000;
pub(super) const FORMULA_VERSION: &str = "base_13_node_linear_v1";

pub(super) fn charge_applicability(
    subject: ResolvedSubnetSubject,
    kind: SubnetKind,
) -> (bool, String) {
    match kind {
        SubnetKind::Application | SubnetKind::CloudEngine => {
            (true, "charged_user_canister_subnet".to_string())
        }
        SubnetKind::System if subject == ResolvedSubnetSubject::Subnet => {
            (false, "system_subnet_core_canister".to_string())
        }
        SubnetKind::System => (false, "system_subnet_unknown_subject".to_string()),
        SubnetKind::Unknown => (false, "unknown_subnet_type".to_string()),
    }
}

pub(super) fn catalog_cycles_per_billion(subnet: &SubnetInfo) -> Option<u128> {
    if !subnet.subnet_kind.charges_apply_by_default() {
        return None;
    }
    let node_count = u128::from(subnet.node_count?);
    if node_count == 0 {
        return None;
    }
    Some((BASE_13_NODE_CYCLES_PER_BILLION_INSTRUCTIONS * node_count).div_ceil(13))
}
