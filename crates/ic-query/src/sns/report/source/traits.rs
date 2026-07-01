//! Module: sns::report::source::traits
//!
//! Responsibility: group SNS report source contracts.
//! Does not own: live transport, cache IO, report assembly, or rendering.
//! Boundary: defines source contracts used by report builders and tests.

use crate::sns::report::{
    MainnetSns, MainnetSnsList, MainnetSnsNeuronPage, MainnetSnsNeurons, MainnetSnsProposal,
    MainnetSnsProposalPage, MainnetSnsProposals, MainnetSnsToken, SnsGovernanceParameters,
    SnsHostError, SnsNeuronId, SnsProposalTopicFilter, SnsSourceRequest,
};

///
/// SnsListSource
///
/// Source contract for fetching deployed SNS inventory.
///

pub trait SnsListSource {
    /// Fetch deployed SNS instances for one source endpoint and network.
    fn fetch_deployed_snses(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsList, SnsHostError>;
}

///
/// SnsNeuronsSource
///
/// Source contract for fetching bounded and paged SNS neuron data.
///

pub(in crate::sns::report) trait SnsNeuronsSource: SnsListSource {
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
/// SnsParamsSource
///
/// Source contract for fetching governance parameters for one deployed SNS.
///

pub trait SnsParamsSource: SnsListSource {
    /// Fetch SNS governance parameters for one resolved SNS.
    fn fetch_sns_params(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError>;
}

///
/// SnsProposalSource
///
/// Source contract for fetching one SNS proposal by id.
///

pub(in crate::sns::report) trait SnsProposalSource: SnsListSource {
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

pub(in crate::sns::report) trait SnsProposalsSource: SnsListSource {
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

pub trait SnsTokenSource: SnsListSource {
    /// Fetch SNS ledger token metadata for one resolved SNS.
    fn fetch_sns_token(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError>;
}
