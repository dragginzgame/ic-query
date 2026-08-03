//! Module: sns::report::source::traits
//!
//! Responsibility: group SNS report source contracts.
//! Does not own: live transport, cache IO, report assembly, or rendering.
//! Boundary: defines source contracts used by report builders and tests.

use crate::sns::report::{
    MainnetSns, MainnetSnsCanisterInventory, MainnetSnsCanisters, MainnetSnsInventory,
    MainnetSnsMetadata, MainnetSnsMetrics, MainnetSnsNeuron, MainnetSnsNeuronPage,
    MainnetSnsNeurons, MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals,
    MainnetSnsRewardNeuronPage, MainnetSnsSwap, MainnetSnsToken, MainnetSnsUpgrade,
    SnsGovernanceParameters, SnsHostError, SnsNeuronId, SnsProposalTopicFilter, SnsRewardEvent,
    SnsRunningVersionResponse, SnsSourceRequest,
};

///
/// SnsDiscoverySource
///
/// Source contract for fetching deployed SNS inventory and explicit metadata targets.
///

pub trait SnsDiscoverySource {
    /// Fetch the unenriched deployed-SNS inventory for one source endpoint and network.
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError>;

    /// Fetch metadata for exactly the supplied deployed-SNS targets.
    fn fetch_sns_metadata(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError>;
}

///
/// SnsCanisterSource
///
/// Source contract for fetching SNS Root canister inventory and health.
///

pub trait SnsCanisterSource: SnsDiscoverySource {
    /// Fetch Root inventory and operational health for one resolved SNS.
    fn fetch_sns_canisters(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsCanisterInventory, SnsHostError>;
}

///
/// SnsNeuronSource
///
/// Source contract for fetching one exact full SNS neuron detail.
///

pub trait SnsNeuronSource: SnsDiscoverySource {
    /// Fetch exactly one full native SNS Governance neuron.
    fn fetch_sns_neuron(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        neuron_id: &str,
    ) -> Result<MainnetSnsNeuron, SnsHostError>;
}

///
/// SnsNeuronsSource
///
/// Source contract for fetching bounded and paged SNS neuron data.
///

pub trait SnsNeuronsSource: SnsDiscoverySource {
    /// Fetch a bounded SNS neuron listing for one resolved SNS.
    fn fetch_sns_neurons(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError>;

    /// Fetch one SNS neuron page for complete snapshot refresh.
    fn fetch_sns_neuron_page(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError>;
}

///
/// SnsRewardSource
///
/// Source contract for bracketed API-exhausted SNS reward checkpoint collection.
///

pub trait SnsRewardSource: SnsDiscoverySource {
    /// Fetch the complete native running-version response for one bracket position.
    fn fetch_sns_reward_running_version(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<SnsRunningVersionResponse, SnsHostError>;

    /// Fetch the complete native nervous-system parameters for one bracket position.
    fn fetch_sns_reward_parameters(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError>;

    /// Fetch the complete latest reward event for one bracket position.
    fn fetch_sns_reward_event(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<SnsRewardEvent, SnsHostError>;

    /// Fetch one strict full-evidence neuron page using an exclusive native cursor.
    fn fetch_sns_reward_neuron_page(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
    ) -> Result<MainnetSnsRewardNeuronPage, SnsHostError>;
}

///
/// SnsParamsSource
///
/// Source contract for fetching governance parameters for one deployed SNS.
///

pub trait SnsParamsSource: SnsDiscoverySource {
    /// Fetch SNS governance parameters for one resolved SNS.
    fn fetch_sns_params(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError>;
}

///
/// SnsSwapSource
///
/// Source contract for bounded lifecycle and sale-state queries against one deployed SNS.
///

pub trait SnsSwapSource: SnsDiscoverySource {
    /// Fetch three bounded native swap query components for one resolved SNS.
    fn fetch_sns_swap(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsSwap, SnsHostError>;
}

///
/// SnsMetricsSource
///
/// Source contract for one bounded SNS Governance metrics query.
///

pub trait SnsMetricsSource: SnsDiscoverySource {
    /// Fetch native Governance metrics for one resolved SNS and proposal window.
    fn fetch_sns_metrics(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        time_window_seconds: u64,
    ) -> Result<MainnetSnsMetrics, SnsHostError>;
}

///
/// SnsUpgradeSource
///
/// Source contract for bounded native upgrade-version queries for one deployed SNS.
///

pub trait SnsUpgradeSource: SnsDiscoverySource {
    /// Fetch the running Governance version and next blessed SNS-W version.
    fn fetch_sns_upgrade(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsUpgrade, SnsHostError>;
}

///
/// SnsProposalSource
///
/// Source contract for fetching one SNS proposal by id.
///

pub trait SnsProposalSource: SnsDiscoverySource {
    /// Fetch one SNS governance proposal for one resolved SNS.
    fn fetch_sns_proposal(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        proposal_id: u64,
    ) -> Result<MainnetSnsProposal, SnsHostError>;
}

///
/// SnsProposalsSource
///
/// Source contract for fetching bounded SNS proposal listings.
///

pub trait SnsProposalsSource: SnsDiscoverySource {
    /// Fetch a bounded SNS governance proposal page for one resolved SNS.
    fn fetch_sns_proposals(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        topic: SnsProposalTopicFilter,
    ) -> Result<MainnetSnsProposals, SnsHostError>;

    /// Fetch one unfiltered SNS governance proposal page for snapshot refresh.
    fn fetch_sns_proposal_page(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> Result<MainnetSnsProposalPage, SnsHostError>;
}

///
/// SnsTokenSource
///
/// Source contract for fetching token metadata for one deployed SNS.
///

pub trait SnsTokenSource: SnsDiscoverySource {
    /// Fetch SNS ledger token metadata for one resolved SNS.
    fn fetch_sns_token(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError>;
}
