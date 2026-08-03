use crate::nns::topology::report::{NnsTopologyProviderRow, NnsTopologyProviderStatus};

pub(super) fn sort_provider_rows(providers: &mut [NnsTopologyProviderRow]) {
    providers.sort_by(|left, right| {
        (
            left.status.sort_rank(),
            std::cmp::Reverse(left.topology_node_count),
            left.node_provider_principal.as_str(),
        )
            .cmp(&(
                right.status.sort_rank(),
                std::cmp::Reverse(right.topology_node_count),
                right.node_provider_principal.as_str(),
            ))
    });
}

pub(super) const fn provider_status(
    registered: bool,
    topology_node_count: u64,
    node_operator_count: u64,
    over_assigned_node_count: u64,
) -> NnsTopologyProviderStatus {
    if !registered {
        return NnsTopologyProviderStatus::UnknownProvider;
    }
    if over_assigned_node_count > 0 {
        return NnsTopologyProviderStatus::Over;
    }
    if topology_node_count == 0 && node_operator_count == 0 {
        return NnsTopologyProviderStatus::Unused;
    }
    NnsTopologyProviderStatus::Ok
}
