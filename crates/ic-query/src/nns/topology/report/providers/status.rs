use crate::nns::topology::report::NnsTopologyProviderRow;

pub(super) fn sort_provider_rows(providers: &mut [NnsTopologyProviderRow]) {
    providers.sort_by(|left, right| {
        (
            provider_status_rank(&left.status),
            std::cmp::Reverse(left.topology_node_count),
            left.node_provider_principal.as_str(),
        )
            .cmp(&(
                provider_status_rank(&right.status),
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
) -> &'static str {
    if !registered {
        return "unknown_provider";
    }
    if over_assigned_node_count > 0 {
        return "over";
    }
    if topology_node_count == 0 && node_operator_count == 0 {
        return "unused";
    }
    "ok"
}

fn provider_status_rank(status: &str) -> u8 {
    match status {
        "unknown_provider" => 0,
        "over" => 1,
        "unused" => 2,
        "ok" => 3,
        _ => 4,
    }
}
