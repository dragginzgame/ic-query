use super::{fixtures::*, *};
use crate::nns::node::NnsNodeHostError;

#[test]
fn topology_reads_preserve_dependencies_and_stop_at_the_first_source_failure() {
    type Build =
        fn(&NnsTopologyReadRequest, &dyn NnsTopologySource) -> Result<(), NnsTopologyHostError>;
    let all = &[
        "subnet_catalog",
        "nodes",
        "node_providers",
        "node_operators",
        "data_centers",
    ][..];
    let inventory = &all[1..];
    let builders: [(Build, &[&str]); 8] = [
        (
            |r, s| build_nns_topology_summary_report_with_source(r, s).map(|_| ()),
            all,
        ),
        (
            |r, s| build_nns_topology_versions_report_with_source(r, s).map(|_| ()),
            all,
        ),
        (
            |r, s| build_nns_topology_coverage_report_with_source(r, s).map(|_| ()),
            all,
        ),
        (
            |r, s| build_nns_topology_check_report_with_source(r, s).map(|_| ()),
            all,
        ),
        (
            |r, s| build_nns_topology_gaps_report_with_source(r, s).map(|_| ()),
            inventory,
        ),
        (
            |r, s| build_nns_topology_providers_report_with_source(r, s).map(|_| ()),
            inventory,
        ),
        (
            |r, s| build_nns_topology_capacity_report_with_source(r, s).map(|_| ()),
            &["node_operators"],
        ),
        (
            |r, s| build_nns_topology_regions_report_with_source(r, s).map(|_| ()),
            &["data_centers"],
        ),
    ];

    for (build, expected) in builders {
        let request = topology_read_request_fixture();
        let source = RecordingTopologySource::default();
        build(&request, &source).expect("topology read");
        assert_eq!(*source.calls.borrow(), expected);

        for (index, component) in expected.iter().enumerate() {
            let source = RecordingTopologySource {
                fail_at: Some(component),
                ..RecordingTopologySource::default()
            };
            let error = build(&request, &source).expect_err("source failure must propagate");
            assert!(matches!(
                error,
                NnsTopologyHostError::Node(NnsNodeHostError::InvalidSourceData { reason })
                    if reason == *component
            ));
            assert_eq!(*source.calls.borrow(), expected[..=index]);
        }

        let mut request = request;
        request.network = "local".to_string();
        let source = RecordingTopologySource::default();
        assert!(matches!(
            build(&request, &source),
            Err(NnsTopologyHostError::UnsupportedNetwork { network }) if network == "local"
        ));
        assert!(source.calls.borrow().is_empty());
    }
}
