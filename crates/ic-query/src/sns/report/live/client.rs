//! Module: sns::report::live::client
//!
//! Responsibility: live SNS source adapter root.
//! Does not own: report assembly, cache IO, command parsing, or rendering.
//! Boundary: implements source traits by delegating to live fetch helpers.

use crate::report::CollectionSourceRequest;
use crate::sns::report::{
    MainnetSns, MainnetSnsCanisterInventory, MainnetSnsCanisters, MainnetSnsInventory,
    MainnetSnsLifecycle, MainnetSnsMetadata, MainnetSnsMetrics, MainnetSnsNeuronPage,
    MainnetSnsRewardNeuronPage, MainnetSnsSwap, MainnetSnsToken, MainnetSnsUpgrade,
    SnsCanisterSource, SnsCatalogSource, SnsDiscoverySource, SnsGovernanceParameters, SnsHostError,
    SnsMetricsSource, SnsNeuronDetail, SnsNeuronId, SnsNeuronRow, SnsNeuronSource,
    SnsNeuronsSource, SnsParamsSource, SnsProposalRow, SnsProposalSource, SnsProposalTopicFilter,
    SnsProposalsSource, SnsRewardEvent, SnsRewardSource, SnsRunningVersionResponse, SnsSwapSource,
    SnsTokenSource, SnsUpgradeSource,
    live::fetch::{
        fetch_mainnet_sns_canisters, fetch_mainnet_sns_inventory, fetch_mainnet_sns_lifecycles,
        fetch_mainnet_sns_metadata, fetch_mainnet_sns_metrics, fetch_mainnet_sns_neuron,
        fetch_mainnet_sns_neuron_page, fetch_mainnet_sns_neurons, fetch_mainnet_sns_params,
        fetch_mainnet_sns_proposal, fetch_mainnet_sns_proposal_page, fetch_mainnet_sns_proposals,
        fetch_mainnet_sns_reward_event, fetch_mainnet_sns_reward_neuron_page,
        fetch_mainnet_sns_running_version, fetch_mainnet_sns_swap, fetch_mainnet_sns_token,
        fetch_mainnet_sns_upgrade,
    },
};

///
/// LiveSnsSource
///
/// Live mainnet SNS source used by report builders outside tests.
///

#[derive(Clone, Debug, Default)]
pub struct LiveSnsSource {
    pub(crate) agent: ic_host_tools::agent::EndpointAgent,
}

impl SnsDiscoverySource for LiveSnsSource {
    fn fetch_sns_inventory<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsInventory, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_inventory(self, request))
    }

    fn fetch_sns_metadata<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        targets: &'a [MainnetSnsCanisters],
    ) -> crate::report::SourceFuture<'a, Result<Vec<MainnetSnsMetadata>, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_metadata(self, request, targets))
    }
}

impl SnsCatalogSource for LiveSnsSource {
    fn fetch_sns_lifecycles<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        targets: &'a [MainnetSnsCanisters],
    ) -> crate::report::SourceFuture<'a, Result<Vec<MainnetSnsLifecycle>, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_lifecycles(self, request, targets))
    }
}

impl SnsCanisterSource for LiveSnsSource {
    fn fetch_sns_canisters<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsCanisterInventory, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_canisters(self, request, sns))
    }
}

impl SnsNeuronSource for LiveSnsSource {
    fn fetch_sns_neuron<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        neuron_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<SnsNeuronDetail, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_neuron(self, request, sns, neuron_id))
    }
}

impl SnsTokenSource for LiveSnsSource {
    fn fetch_sns_token<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsToken, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_token(self, request, sns))
    }
}

impl SnsParamsSource for LiveSnsSource {
    fn fetch_sns_params<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<SnsGovernanceParameters, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_params(self, request, sns))
    }
}

impl SnsSwapSource for LiveSnsSource {
    fn fetch_sns_swap<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsSwap, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_swap(self, request, sns))
    }
}

impl SnsMetricsSource for LiveSnsSource {
    fn fetch_sns_metrics<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        time_window_seconds: u64,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsMetrics, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_metrics(
            self,
            request,
            sns,
            time_window_seconds,
        ))
    }
}

impl SnsUpgradeSource for LiveSnsSource {
    fn fetch_sns_upgrade<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsUpgrade, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_upgrade(self, request, sns))
    }
}

impl SnsProposalSource for LiveSnsSource {
    fn fetch_sns_proposal<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        proposal_id: u64,
    ) -> crate::report::SourceFuture<'a, Result<SnsProposalRow, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_proposal(self, request, sns, proposal_id))
    }
}

impl SnsProposalsSource for LiveSnsSource {
    fn fetch_sns_proposals<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &'a [i32],
        topic: SnsProposalTopicFilter,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_proposals(
            self,
            request,
            sns,
            limit,
            before_proposal_id,
            include_status,
            topic,
        ))
    }

    fn fetch_sns_proposal_page<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_proposal_page(
            self,
            request,
            sns,
            limit,
            before_proposal_id,
        ))
    }
}

impl SnsNeuronsSource for LiveSnsSource {
    fn fetch_sns_neurons<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        owner_principal_id: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsNeuronRow>, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_neurons(
            self,
            request,
            sns,
            limit,
            owner_principal_id,
        ))
    }

    fn fetch_sns_neuron_page<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        start_page_at: Option<&'a SnsNeuronId>,
        owner_principal_id: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsNeuronPage, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_neuron_page(
            self,
            request,
            sns,
            limit,
            start_page_at,
            owner_principal_id,
        ))
    }
}

impl SnsRewardSource for LiveSnsSource {
    fn fetch_sns_reward_running_version<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<SnsRunningVersionResponse, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_running_version(self, request, sns))
    }

    fn fetch_sns_reward_parameters<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<SnsGovernanceParameters, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_params(self, request, sns))
    }

    fn fetch_sns_reward_event<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<SnsRewardEvent, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_reward_event(self, request, sns))
    }

    fn fetch_sns_reward_neuron_page<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        start_page_at: Option<&'a SnsNeuronId>,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsRewardNeuronPage, SnsHostError>> {
        Box::pin(fetch_mainnet_sns_reward_neuron_page(
            self,
            request,
            sns,
            limit,
            start_page_at,
        ))
    }
}
