use super::{
    CLOUD_ENGINE_NODE_INCLUDED_STATUSES, CLOUD_ENGINE_NODE_REWARD_TYPE, CloudEngineNodeInfoReport,
    CloudEngineNodeListReport, CloudEngineNodeRow, cloud_engine_node_info_report_text,
    cloud_engine_node_list_report_text,
};
use crate::ic::{IcDashboardReportProvenance, IcNodeStatusCounts};

const NODE_A: &str = "53amq-7hjxu-6lxaj-o2sp6-kmngy-qa22h-b7bo6-oeyyn-fkqnv-7tauf-7qe";
const NODE_B: &str = "72xg3-cvfed-jrbv3-kal7e-o53wl-tug5y-t432v-5ovop-7et6y-abuzx-oae";
const PROVIDER: &str = "bvcsg-3od6r-jnydw-eysln-aql7w-td5zn-ay5m6-sibd2-jzojt-anwag-mqe";
const OPERATOR: &str = "e3aue-mkha2-6zddy-xbmd7-3oybi-3nfoh-3bwgn-izbjn-uuqx2-ykc2z-7qe";
const CLOUD_ENGINE_SUBNET: &str = "nx5oj-b2azr-x3alh-sgf7i-duhfw-bflus-hisa2-5n2oq-tv7sd-haspd-cae";

#[test]
fn text_separates_dashboard_provenance_from_type4_nodes() {
    let node = node(NODE_A, "UP");
    let list = CloudEngineNodeListReport {
        provenance: provenance(),
        node_reward_type: CLOUD_ENGINE_NODE_REWARD_TYPE.to_string(),
        included_statuses: included_statuses(),
        requested_node_provider_id: None,
        node_count: 1,
        status_counts: IcNodeStatusCounts {
            total: 1,
            up: 1,
            ..IcNodeStatusCounts::default()
        },
        node_provider_count: 1,
        cloud_engine_subnet_count: 1,
        unassigned_cloud_engine_node_count: 0,
        nodes: vec![node.clone()],
    };
    let info = CloudEngineNodeInfoReport {
        provenance: provenance(),
        node,
    };

    let list_text = cloud_engine_node_list_report_text(&list);
    let info_text = cloud_engine_node_info_report_text(&info);
    assert!(list_text.contains("node_count: 1\n"));
    assert!(list_text.contains("unassigned_cloud_engine_node_count: 0\n\nCloudEngine nodes\n"));
    assert!(info_text.contains("node_reward_type: Type4"));
    assert!(info_text.contains("cloud_engine_subnet_id: nx5oj"));
}

fn provenance() -> IcDashboardReportProvenance {
    IcDashboardReportProvenance {
        schema_version: 1,
        network: "ic".to_string(),
        authority: "official_ic_dashboard_api".to_string(),
        source_endpoint: "https://ic-api.internetcomputer.org/api/v3".to_string(),
        fetched_at: "2026-08-08T12:00:00Z".to_string(),
        fetched_by: "test".to_string(),
        certified: false,
        point_in_time_guaranteed: false,
    }
}

fn included_statuses() -> Vec<String> {
    CLOUD_ENGINE_NODE_INCLUDED_STATUSES
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn node(node_id: &str, status: &str) -> CloudEngineNodeRow {
    CloudEngineNodeRow {
        node_id: node_id.to_string(),
        node_operator_id: OPERATOR.to_string(),
        node_provider_id: PROVIDER.to_string(),
        node_provider_name: "DFINITY Stiftung".to_string(),
        node_type: "UNASSIGNED".to_string(),
        node_reward_type: CLOUD_ENGINE_NODE_REWARD_TYPE.to_string(),
        status: status.to_string(),
        alert_name: None,
        subnet_id: None,
        cloud_engine_subnet_id: Some(CLOUD_ENGINE_SUBNET.to_string()),
        data_center_id: "tp1".to_string(),
        data_center_name: "Tampa".to_string(),
        owner: "Flexential".to_string(),
        region: "North America,US,Florida".to_string(),
        guestos_version: Some("release".to_string()),
        guestos_tee_active: Some(false),
        ip_address: Some("2001:db8::1".to_string()),
        ipv4_connectivity_status: Some(false),
        node_hardware_generation: Some("Gen1".to_string()),
    }
}

#[cfg(feature = "dashboard-host")]
mod host {
    use super::*;
    use crate::{
        cloud_engine::{
            CloudEngineNodeInfoRequest, CloudEngineNodeInfoSourceData, CloudEngineNodeListRequest,
            CloudEngineNodeListSourceData, CloudEngineNodeSource,
            DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT,
            build_cloud_engine_node_info_report_with_source,
            build_cloud_engine_node_list_report_with_source,
        },
        ic::{IcHostError, IcSourceRequest},
    };
    use std::cell::Cell;

    const NOW: u64 = 1_800_000_000;

    #[test]
    fn list_validates_scope_sorts_rows_and_counts_status_and_assignment() {
        let source = Fixture::default();
        let report = build_cloud_engine_node_list_report_with_source(&list_request(None), &source)
            .expect("Type4 nodes");

        assert_eq!(source.list_calls.get(), 1);
        assert_eq!(report.node_count, 2);
        assert_eq!(report.nodes[0].node_id, NODE_A);
        assert_eq!(report.nodes[1].node_id, NODE_B);
        assert_eq!(report.status_counts.up, 1);
        assert_eq!(report.status_counts.down, 1);
        assert_eq!(report.node_provider_count, 1);
        assert_eq!(report.cloud_engine_subnet_count, 1);
        assert_eq!(report.unassigned_cloud_engine_node_count, 1);
    }

    #[test]
    fn provider_filter_is_canonicalized_before_source_and_enforced_on_rows() {
        let source = Fixture::default();
        let report =
            build_cloud_engine_node_list_report_with_source(&list_request(Some(PROVIDER)), &source)
                .expect("provider Type4 nodes");
        assert_eq!(report.requested_node_provider_id.as_deref(), Some(PROVIDER));
        assert_eq!(source.seen_provider.get(), Some(PROVIDER));

        source.mutation.set(Some(Mutation::WrongProvider));
        assert!(matches!(
            build_cloud_engine_node_list_report_with_source(&list_request(Some(PROVIDER)), &source),
            Err(IcHostError::InvalidSourceData { .. })
        ));
    }

    #[test]
    fn exact_info_requires_the_requested_type4_node() {
        let source = Fixture::default();
        let request = CloudEngineNodeInfoRequest::new(
            "ic",
            DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT,
            NOW,
            NODE_A,
        );
        let report = build_cloud_engine_node_info_report_with_source(&request, &source)
            .expect("exact Type4 node");
        assert_eq!(report.node.node_id, NODE_A);

        source.mutation.set(Some(Mutation::WrongRewardType));
        assert!(matches!(
            build_cloud_engine_node_info_report_with_source(&request, &source),
            Err(IcHostError::InvalidSourceData { .. })
        ));
    }

    #[test]
    fn list_source_contract_rejects_inconsistent_scope_and_duplicate_rows() {
        let source = Fixture::default();
        for mutation in [
            Mutation::WrongSource,
            Mutation::WrongRewardType,
            Mutation::WrongStatuses,
            Mutation::DuplicateNode,
            Mutation::UnknownListStatus,
        ] {
            source.mutation.set(Some(mutation));
            assert!(matches!(
                build_cloud_engine_node_list_report_with_source(&list_request(None), &source),
                Err(IcHostError::InvalidSourceData { .. })
            ));
        }
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        WrongSource,
        WrongRewardType,
        WrongStatuses,
        DuplicateNode,
        UnknownListStatus,
        WrongProvider,
    }

    #[derive(Default)]
    struct Fixture {
        list_calls: Cell<usize>,
        mutation: Cell<Option<Mutation>>,
        seen_provider: Cell<Option<&'static str>>,
    }

    impl CloudEngineNodeSource for Fixture {
        fn fetch_cloud_engine_node_list(
            &self,
            request: &IcSourceRequest,
            node_provider_id: Option<&str>,
        ) -> Result<CloudEngineNodeListSourceData, IcHostError> {
            self.list_calls.set(self.list_calls.get() + 1);
            self.seen_provider.set(node_provider_id.map(|_| PROVIDER));
            let mut data = CloudEngineNodeListSourceData {
                source: request.clone(),
                requested_node_provider_id: node_provider_id.map(str::to_string),
                node_reward_type: CLOUD_ENGINE_NODE_REWARD_TYPE.to_string(),
                included_statuses: included_statuses(),
                nodes: vec![
                    {
                        let mut row = node(NODE_B, "DOWN");
                        row.cloud_engine_subnet_id = None;
                        row
                    },
                    node(NODE_A, "UP"),
                ],
            };
            match self.mutation.take() {
                Some(Mutation::WrongSource) => data.source.endpoint.push_str("/other"),
                Some(Mutation::WrongRewardType) => {
                    data.node_reward_type = "Type3dot1".to_string();
                    data.nodes[0].node_reward_type = "Type3dot1".to_string();
                }
                Some(Mutation::WrongStatuses) => {
                    data.included_statuses.pop();
                }
                Some(Mutation::DuplicateNode) => {
                    data.nodes[1].node_id = data.nodes[0].node_id.clone();
                }
                Some(Mutation::UnknownListStatus) => {
                    data.nodes[0].status = "FUTURE".to_string();
                }
                Some(Mutation::WrongProvider) => {
                    data.nodes[0].node_provider_id = "aaaaa-aa".to_string();
                }
                None => {}
            }
            Ok(data)
        }

        fn fetch_cloud_engine_node_info(
            &self,
            request: &IcSourceRequest,
            node_id: &str,
        ) -> Result<CloudEngineNodeInfoSourceData, IcHostError> {
            let mut row = node(node_id, "UP");
            if matches!(self.mutation.take(), Some(Mutation::WrongRewardType)) {
                row.node_reward_type = "Type3dot1".to_string();
            }
            Ok(CloudEngineNodeInfoSourceData {
                source: request.clone(),
                node_id: node_id.to_string(),
                node: row,
            })
        }
    }

    fn list_request(node_provider_id: Option<&str>) -> CloudEngineNodeListRequest {
        let request = CloudEngineNodeListRequest::new(
            "ic",
            DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT,
            NOW,
        );
        match node_provider_id {
            Some(provider) => request.with_node_provider_id(provider),
            None => request,
        }
    }
}
