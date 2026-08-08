use super::{
    IC_API_BOUNDARY_NODE_REPORT_SCHEMA_VERSION, IcApiBoundaryNodeReport, IcApiBoundaryNodeRequest,
    IcApiBoundaryNodeRow, IcCertifiedStateProvenance, ic_api_boundary_node_report_text,
};

const NODE_A: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";
const NODE_B: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
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

#[cfg(feature = "ic-state-host")]
mod host {
    use super::*;
    use crate::{
        ic::{
            DEFAULT_IC_STATE_SOURCE_ENDPOINT, IcApiBoundaryNodeHostError, IcApiBoundaryNodeSource,
            IcApiBoundaryNodeSourceData, IcApiBoundaryNodeSourceRequest,
            build_ic_api_boundary_node_report_with_source,
        },
        subnet_catalog::format_utc_timestamp_secs,
    };
    use candid::Principal;
    use ic_agent::{
        Certificate,
        hash_tree::{HashTree, fork, label, leaf},
    };
    use std::cell::Cell;

    #[test]
    fn custom_source_preserves_certificate_time_and_canonicalizes_rows() {
        let source = FixtureSource::default();
        let report = build_ic_api_boundary_node_report_with_source(&request(), &source)
            .expect("certified API boundary nodes");

        assert_eq!(source.calls.get(), 1);
        assert_eq!(report.node_count, 2);
        assert_eq!(report.rows[0].node_id, NODE_A);
        assert_eq!(report.rows[1].node_id, NODE_B);
        assert_eq!(
            report.provenance.certificate_time_unix_nanos,
            NOW * 1_000_000_000
        );
        assert_eq!(report.provenance.certificate_time_unix_seconds, NOW);
        assert!(report.provenance.certified);
        assert!(report.provenance.point_in_time_guaranteed);
    }

    #[test]
    fn custom_source_contract_rejects_inconsistent_or_invalid_evidence() {
        for mutation in [
            Mutation::WrongSource,
            Mutation::StaleCertificate,
            Mutation::Empty,
            Mutation::DuplicateDomain,
            Mutation::InvalidIpv4,
            Mutation::InvalidIpv6,
            Mutation::InvalidNode,
        ] {
            let source = FixtureSource {
                mutation: Cell::new(Some(mutation)),
                ..FixtureSource::default()
            };
            let error = build_ic_api_boundary_node_report_with_source(&request(), &source)
                .expect_err("invalid source evidence must fail");

            assert!(matches!(
                error,
                IcApiBoundaryNodeHostError::InvalidSourceData { .. }
            ));
        }
    }

    #[test]
    fn authenticated_tree_decoder_preserves_node_ids_and_optional_ipv4() {
        let source_request = source_request();
        let certificate = Certificate {
            tree: fork(
                label(
                    "api_boundary_nodes",
                    certified_nodes(vec![
                        (NODE_B, "api2.example.com", Some("192.0.2.2"), "2001:db8::2"),
                        (NODE_A, "api1.example.com", None, "2001:db8::1"),
                    ]),
                ),
                label("time", leaf(encode_unsigned_leb128(NOW * 1_000_000_000))),
            ),
            signature: Vec::new(),
            delegation: None,
        };

        let data = super::super::host::source_data_from_certificate(&source_request, certificate)
            .expect("authenticated tree shape");

        assert_eq!(data.certificate_time_unix_nanos, NOW * 1_000_000_000);
        assert_eq!(data.rows.len(), 2);
        assert_eq!(data.rows[0].node_id, NODE_A);
        assert_eq!(data.rows[0].ipv4_address, None);
        assert_eq!(data.rows[1].node_id, NODE_B);
        assert_eq!(data.rows[1].ipv4_address.as_deref(), Some("192.0.2.2"));
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Mutation {
        WrongSource,
        StaleCertificate,
        Empty,
        DuplicateDomain,
        InvalidIpv4,
        InvalidIpv6,
        InvalidNode,
    }

    #[derive(Default)]
    struct FixtureSource {
        calls: Cell<usize>,
        mutation: Cell<Option<Mutation>>,
    }

    impl IcApiBoundaryNodeSource for FixtureSource {
        fn fetch_api_boundary_nodes(
            &self,
            request: &IcApiBoundaryNodeSourceRequest,
        ) -> Result<IcApiBoundaryNodeSourceData, IcApiBoundaryNodeHostError> {
            self.calls.set(self.calls.get() + 1);
            let mut data = IcApiBoundaryNodeSourceData {
                source: request.clone(),
                certificate_time_unix_nanos: NOW * 1_000_000_000,
                rows: vec![
                    row(NODE_B, "api2.example.com", Some("192.0.2.2"), "2001:db8::2"),
                    row(NODE_A, "api1.example.com", None, "2001:db8::1"),
                ],
            };
            match self.mutation.take() {
                Some(Mutation::WrongSource) => data.source.endpoint.push_str("/other"),
                Some(Mutation::StaleCertificate) => {
                    data.certificate_time_unix_nanos = (NOW - 301) * 1_000_000_000;
                }
                Some(Mutation::Empty) => data.rows.clear(),
                Some(Mutation::DuplicateDomain) => {
                    data.rows[1].domain = data.rows[0].domain.clone();
                }
                Some(Mutation::InvalidIpv4) => {
                    data.rows[0].ipv4_address = Some("999.0.2.2".to_string());
                }
                Some(Mutation::InvalidIpv6) => {
                    data.rows[0].ipv6_address = "not-an-ip".to_string();
                }
                Some(Mutation::InvalidNode) => data.rows[0].node_id = "not-a-node".to_string(),
                None => {}
            }
            Ok(data)
        }
    }

    fn request() -> IcApiBoundaryNodeRequest {
        IcApiBoundaryNodeRequest::new(DEFAULT_IC_STATE_SOURCE_ENDPOINT, NOW)
    }

    fn source_request() -> IcApiBoundaryNodeSourceRequest {
        IcApiBoundaryNodeSourceRequest {
            network: "ic".to_string(),
            endpoint: DEFAULT_IC_STATE_SOURCE_ENDPOINT.to_string(),
            effective_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
            observed_at_unix_seconds: NOW,
            fetched_at: format_utc_timestamp_secs(NOW),
            fetched_by: "ic-query".to_string(),
        }
    }

    fn certified_node(
        node_id: &str,
        domain: &str,
        ipv4_address: Option<&str>,
        ipv6_address: &str,
    ) -> HashTree<Vec<u8>> {
        let principal = Principal::from_text(node_id).expect("fixture node id");
        let mut fields = vec![label("domain", leaf(domain.as_bytes().to_vec()))];
        if let Some(ipv4_address) = ipv4_address {
            fields.push(label(
                "ipv4_address",
                leaf(ipv4_address.as_bytes().to_vec()),
            ));
        }
        fields.push(label(
            "ipv6_address",
            leaf(ipv6_address.as_bytes().to_vec()),
        ));
        label(principal.as_slice(), fork_all(fields))
    }

    fn certified_nodes(mut rows: Vec<(&str, &str, Option<&str>, &str)>) -> HashTree<Vec<u8>> {
        rows.sort_unstable_by(|left, right| {
            Principal::from_text(left.0)
                .expect("left fixture node")
                .as_slice()
                .cmp(
                    Principal::from_text(right.0)
                        .expect("right fixture node")
                        .as_slice(),
                )
        });
        fork_all(
            rows.into_iter()
                .map(|(node_id, domain, ipv4, ipv6)| certified_node(node_id, domain, ipv4, ipv6))
                .collect(),
        )
    }

    fn fork_all(mut trees: Vec<HashTree<Vec<u8>>>) -> HashTree<Vec<u8>> {
        let mut tree = trees.pop().expect("nonempty tree fixture");
        while let Some(left) = trees.pop() {
            tree = fork(left, tree);
        }
        tree
    }

    fn encode_unsigned_leb128(mut value: u64) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(10);
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                return bytes;
            }
        }
    }
}
