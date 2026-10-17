//! Module: sns::report::source::traits
//!
//! Responsibility: group SNS report source contracts.
//! Does not own: live transport, cache IO, report assembly, or rendering.
//! Boundary: defines source contracts used by report builders and tests.

use crate::sns::report::{
    MainnetSns, MainnetSnsCanisterInventory, MainnetSnsCanisters, MainnetSnsInventory,
    MainnetSnsLifecycle, MainnetSnsMetadata, MainnetSnsMetrics, MainnetSnsNeuronPage,
    MainnetSnsRewardNeuronPage, MainnetSnsSwap, MainnetSnsToken, MainnetSnsUpgrade,
    SnsGovernanceParameters, SnsHostError, SnsNeuronDetail, SnsNeuronId, SnsNeuronRow,
    SnsProposalRow, SnsProposalTopicFilter, SnsRewardEvent, SnsRunningVersionResponse,
    SnsSourceRequest,
};

///
/// SnsDiscoverySource
///
/// Source contract for fetching deployed SNS inventory and explicit metadata targets.
///

pub trait SnsDiscoverySource: Sync {
    /// Fetch the unenriched deployed-SNS inventory for one source endpoint and network.
    fn fetch_sns_inventory<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsInventory, SnsHostError>>;

    /// Fetch metadata for exactly the supplied deployed-SNS targets.
    fn fetch_sns_metadata<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        targets: &'a [MainnetSnsCanisters],
    ) -> crate::report::SourceFuture<'a, Result<Vec<MainnetSnsMetadata>, SnsHostError>>;
}

///
/// SnsCatalogSource
///
/// Source contract for enriching a complete deployed-SNS catalog with Swap lifecycles.
///

pub trait SnsCatalogSource: SnsDiscoverySource + Sync {
    /// Fetch lifecycle evidence for exactly the supplied deployed-SNS targets.
    fn fetch_sns_lifecycles<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        targets: &'a [MainnetSnsCanisters],
    ) -> crate::report::SourceFuture<'a, Result<Vec<MainnetSnsLifecycle>, SnsHostError>>;
}

///
/// SnsCanisterSource
///
/// Source contract for fetching SNS Root canister inventory and health.
///

pub trait SnsCanisterSource: SnsDiscoverySource + Sync {
    /// Fetch Root inventory and operational health for one resolved SNS.
    fn fetch_sns_canisters<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsCanisterInventory, SnsHostError>>;
}

///
/// SnsNeuronSource
///
/// Source contract for fetching one exact full SNS neuron detail.
///

pub trait SnsNeuronSource: SnsDiscoverySource + Sync {
    /// Fetch exactly one full native SNS Governance neuron.
    fn fetch_sns_neuron<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        neuron_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<SnsNeuronDetail, SnsHostError>>;
}

///
/// SnsNeuronsSource
///
/// Source contract for fetching bounded and paged SNS neuron data.
///

pub trait SnsNeuronsSource: SnsDiscoverySource + Sync {
    /// Fetch a bounded SNS neuron listing for one resolved SNS.
    fn fetch_sns_neurons<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        owner_principal_id: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsNeuronRow>, SnsHostError>>;

    /// Fetch one ascending neuron page for complete snapshot refresh.
    /// A supplied cursor must equal the final row's id; empty pages omit it.
    /// A repeated boundary row is allowed; full pages require an advancing cursor.
    /// Short terminal pages may omit the cursor.
    fn fetch_sns_neuron_page<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        start_page_at: Option<&'a SnsNeuronId>,
        owner_principal_id: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsNeuronPage, SnsHostError>>;
}

///
/// SnsRewardSource
///
/// Source contract for bracketed API-exhausted SNS reward checkpoint collection.
///

pub trait SnsRewardSource: SnsDiscoverySource + Sync {
    /// Fetch the complete native running-version response for one bracket position.
    fn fetch_sns_reward_running_version<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<SnsRunningVersionResponse, SnsHostError>>;

    /// Fetch the complete native nervous-system parameters for one bracket position.
    fn fetch_sns_reward_parameters<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<SnsGovernanceParameters, SnsHostError>>;

    /// Fetch the complete latest reward event for one bracket position.
    fn fetch_sns_reward_event<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<SnsRewardEvent, SnsHostError>>;

    /// Fetch one strict full-evidence neuron page using an exclusive native cursor.
    fn fetch_sns_reward_neuron_page<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        start_page_at: Option<&'a SnsNeuronId>,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsRewardNeuronPage, SnsHostError>>;
}

///
/// SnsParamsSource
///
/// Source contract for fetching governance parameters for one deployed SNS.
///

pub trait SnsParamsSource: SnsDiscoverySource + Sync {
    /// Fetch SNS governance parameters for one resolved SNS.
    fn fetch_sns_params<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<SnsGovernanceParameters, SnsHostError>>;
}

///
/// SnsSwapSource
///
/// Source contract for bounded lifecycle and sale-state queries against one deployed SNS.
///

pub trait SnsSwapSource: SnsDiscoverySource + Sync {
    /// Fetch three bounded native swap query components for one resolved SNS.
    fn fetch_sns_swap<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsSwap, SnsHostError>>;
}

///
/// SnsMetricsSource
///
/// Source contract for one bounded SNS Governance metrics query.
///

pub trait SnsMetricsSource: SnsDiscoverySource + Sync {
    /// Fetch native Governance metrics for one resolved SNS and proposal window.
    fn fetch_sns_metrics<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        time_window_seconds: u64,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsMetrics, SnsHostError>>;
}

///
/// SnsUpgradeSource
///
/// Source contract for bounded native upgrade-version queries for one deployed SNS.
///

pub trait SnsUpgradeSource: SnsDiscoverySource + Sync {
    /// Fetch the running Governance version and next blessed SNS-W version.
    fn fetch_sns_upgrade<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsUpgrade, SnsHostError>>;
}

///
/// SnsProposalSource
///
/// Source contract for fetching one SNS proposal by id.
///

pub trait SnsProposalSource: SnsDiscoverySource + Sync {
    /// Fetch one SNS governance proposal for one resolved SNS.
    fn fetch_sns_proposal<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        proposal_id: u64,
    ) -> crate::report::SourceFuture<'a, Result<SnsProposalRow, SnsHostError>>;
}

///
/// SnsProposalsSource
///
/// Source contract for fetching bounded SNS proposal listings.
///

pub trait SnsProposalsSource: SnsDiscoverySource + Sync {
    /// Fetch a bounded SNS governance proposal page for one resolved SNS.
    fn fetch_sns_proposals<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &'a [i32],
        topic: SnsProposalTopicFilter,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>>;

    /// Fetch one unfiltered SNS governance proposal page for snapshot refresh.
    fn fetch_sns_proposal_page<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>>;
}

///
/// SnsTokenSource
///
/// Source contract for fetching token metadata for one deployed SNS.
///

pub trait SnsTokenSource: SnsDiscoverySource + Sync {
    /// Fetch SNS ledger token metadata for one resolved SNS.
    fn fetch_sns_token<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsToken, SnsHostError>>;
}
