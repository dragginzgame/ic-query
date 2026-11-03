use super::NEURON_A;
use crate::sns::report::tests::*;
use std::path::Path;

pub(in crate::sns::report::tests) fn list_request() -> SnsListRequest {
    SnsListRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        all_lifecycles: false,

        sort: SnsListSort::Id,
    }
}

pub(in crate::sns::report::tests) fn info_request(input: &str) -> SnsLookupRequest {
    SnsLookupRequest::new(
        MAINNET_NETWORK.to_string(),
        DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        1_780_531_200,
        input.to_string(),
    )
    .expect("valid SNS selector")
}

pub(in crate::sns::report::tests) fn token_request(input: &str) -> SnsLookupRequest {
    SnsLookupRequest::new(
        MAINNET_NETWORK.to_string(),
        DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        1_780_531_200,
        input.to_string(),
    )
    .expect("valid SNS selector")
}

pub(in crate::sns::report::tests) fn params_request(input: &str) -> SnsLookupRequest {
    SnsLookupRequest::new(
        MAINNET_NETWORK.to_string(),
        DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        1_780_531_200,
        input.to_string(),
    )
    .expect("valid SNS selector")
}

pub(in crate::sns::report::tests) fn metrics_request(input: &str) -> SnsMetricsRequest {
    SnsMetricsRequest::new(
        crate::sns::SnsLookupRequest::new(
            MAINNET_NETWORK,
            DEFAULT_SNS_SOURCE_ENDPOINT,
            1_780_531_200,
            input,
        )
        .expect("valid SNS selector"),
    )
}

pub(in crate::sns::report::tests) fn swap_request(input: &str) -> SnsLookupRequest {
    SnsLookupRequest::new(
        MAINNET_NETWORK.to_string(),
        DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        1_780_531_200,
        input.to_string(),
    )
    .expect("valid SNS selector")
}

pub(in crate::sns::report::tests) fn upgrade_request(input: &str) -> SnsLookupRequest {
    info_request(input)
}

pub(in crate::sns::report::tests) fn proposal_request(input: &str) -> SnsProposalRequest {
    SnsProposalRequest {
        lookup: crate::sns::SnsLookupRequest::new(
            MAINNET_NETWORK.to_string(),
            DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
            1_780_531_200,
            input.to_string(),
        )
        .expect("valid SNS selector"),
        proposal_id: 42,
        cache_root: None,
    }
}

pub(in crate::sns::report::tests) fn proposals_request(input: &str) -> SnsProposalsRequest {
    SnsProposalsRequest::new(info_request(input), 10)
        .with_before_proposal_id(99)
        .with_status(SnsProposalStatusFilter::Open)
        .unwrap()
        .with_topic(SnsProposalTopicFilter::Governance)
}

pub(in crate::sns::report::tests) fn sns_proposals_refresh_request(
    root: &Path,
    max_pages: Option<u32>,
) -> SnsSnapshotRefreshRequest {
    SnsSnapshotRefreshRequest {
        lookup: crate::sns::SnsLookupRequest::new(
            MAINNET_NETWORK.to_string(),
            DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
            1_780_531_200,
            "1".to_string(),
        )
        .expect("valid SNS selector"),
        cache_root: root.to_path_buf(),
        page_size: 100,
        max_pages,
    }
}

pub(in crate::sns::report::tests) fn neurons_request(input: &str) -> SnsNeuronsRequest {
    SnsNeuronsRequest {
        lookup: crate::sns::SnsLookupRequest::new(
            MAINNET_NETWORK.to_string(),
            DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
            1_780_531_200,
            input.to_string(),
        )
        .expect("valid SNS selector"),
        limit: 10,
        collection: crate::sns::SnsNeuronCollection::Live {
            owner_principal_id: None,
        },
    }
}

pub(in crate::sns::report::tests) fn neuron_request(input: &str) -> SnsNeuronRequest {
    SnsNeuronRequest {
        lookup: crate::sns::SnsLookupRequest::new(
            MAINNET_NETWORK.to_string(),
            DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
            1_780_531_200,
            input.to_string(),
        )
        .expect("valid SNS selector"),
        neuron_id: NEURON_A.to_string(),
    }
}

pub(in crate::sns::report::tests) fn sns_neurons_refresh_request(
    root: &Path,
    max_pages: Option<u32>,
) -> SnsSnapshotRefreshRequest {
    SnsSnapshotRefreshRequest {
        lookup: crate::sns::SnsLookupRequest::new(
            MAINNET_NETWORK.to_string(),
            DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
            1_780_531_200,
            "1".to_string(),
        )
        .expect("valid SNS selector"),
        cache_root: root.to_path_buf(),
        page_size: 2,
        max_pages,
    }
}
