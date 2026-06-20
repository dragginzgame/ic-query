use crate::sns::report::tests::*;
use std::path::Path;

pub(in crate::sns::report::tests) fn list_request(verbose: bool) -> SnsListRequest {
    SnsListRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        verbose,
        sort: SnsListSort::Id,
    }
}

pub(in crate::sns::report::tests) fn info_request(input: &str) -> SnsInfoRequest {
    SnsInfoRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
    }
}

pub(in crate::sns::report::tests) fn token_request(input: &str) -> SnsTokenRequest {
    SnsTokenRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
    }
}

pub(in crate::sns::report::tests) fn params_request(input: &str) -> SnsParamsRequest {
    SnsParamsRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
    }
}

pub(in crate::sns::report::tests) fn proposal_request(input: &str) -> SnsProposalRequest {
    SnsProposalRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
        proposal_id: 42,
        icp_root: None,
        verbose: false,
        show_ballots: true,
    }
}

pub(in crate::sns::report::tests) fn proposals_request(input: &str) -> SnsProposalsRequest {
    SnsProposalsRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
        limit: 10,
        before_proposal_id: Some(99),
        status: SnsProposalStatusFilter::Open,
        topic: SnsProposalTopicFilter::Governance,
        eligibility: SnsProposalEligibilityFilter::Any,
        proposer_neuron_id: None,
        sort: SnsProposalsSort::Api,
        sort_direction: SnsProposalSortDirection::Desc,
        icp_root: None,
        verbose: false,
    }
}

pub(in crate::sns::report::tests) fn sns_proposals_refresh_request(
    root: &Path,
    max_pages: Option<u32>,
) -> SnsProposalsRefreshRequest {
    SnsProposalsRefreshRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: "1".to_string(),
        icp_root: root.to_path_buf(),
        page_size: 100,
        max_pages,
    }
}

pub(in crate::sns::report::tests) fn neurons_request(input: &str) -> SnsNeuronsRequest {
    SnsNeuronsRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
        limit: 10,
        owner_principal_id: None,
        sort: SnsNeuronsSort::Api,
        icp_root: None,
        verbose: false,
    }
}

pub(in crate::sns::report::tests) fn sns_neurons_refresh_request(
    root: &Path,
    max_pages: Option<u32>,
) -> SnsNeuronsRefreshRequest {
    SnsNeuronsRefreshRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: "1".to_string(),
        icp_root: root.to_path_buf(),
        page_size: 2,
        max_pages,
    }
}
