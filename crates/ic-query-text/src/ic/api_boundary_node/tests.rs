use crate::ic::ic_api_boundary_node_report_text;
use ic_query::ic::IC_API_BOUNDARY_NODE_REPORT_SCHEMA_VERSION;
use ic_query::ic::IcApiBoundaryNodeReport;
use ic_query::ic::IcApiBoundaryNodeRow;
use ic_query::ic::IcCertifiedStateProvenance;

const NODE_A: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";
const NOW: u64 = 1_800_000_000;

#[test]
fn text_separates_certified_provenance_from_complete_node_table() {
    let report = IcApiBoundaryNodeReport {
        provenance: IcCertifiedStateProvenance {
            schema_version: IC_API_BOUNDARY_NODE_REPORT_SCHEMA_VERSION,
            network: "ic".to_string(),
            authority: "certified_ic_state_tree".to_string(),
            source_endpoint: "https://icp-api.io".to_string(),
            effective_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
            fetched_at_unix_seconds: NOW,
            fetched_at: "2027-01-15T08:00:00Z".to_string(),
            fetched_by: "test".to_string(),
            certificate_time_unix_nanos: NOW * 1_000_000_000,
            certificate_time_unix_seconds: NOW,
            certificate_time: "2027-01-15T08:00:00Z".to_string(),
            certified: true,
            point_in_time_guaranteed: true,
        },
        node_count: 1,
        rows: vec![row(NODE_A, "api1.example.com", None, "2001:db8::1")],
    };

    let text = ic_api_boundary_node_report_text(&report);

    assert!(text.contains("certificate_time_unix_seconds: 1800000000"));
    assert!(text.contains("certified: yes"));
    assert!(text.contains("point_in_time_guaranteed: yes"));
    assert!(text.contains("node_count: 1\n\napi_boundary_nodes:\n"));
    assert!(text.contains("NODE ID"));
    assert!(text.contains("api1.example.com"));
}

fn row(
    node_id: &str,
    domain: &str,
    ipv4_address: Option<&str>,
    ipv6_address: &str,
) -> IcApiBoundaryNodeRow {
    IcApiBoundaryNodeRow {
        node_id: node_id.to_string(),
        domain: domain.to_string(),
        ipv4_address: ipv4_address.map(str::to_string),
        ipv6_address: ipv6_address.to_string(),
    }
}
