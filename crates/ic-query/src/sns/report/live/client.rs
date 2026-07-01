//! Module: sns::report::live::client
//!
//! Responsibility: live SNS source adapter root.
//! Does not own: report assembly, cache IO, command parsing, or rendering.
//! Boundary: implements source traits by delegating to live fetch helpers.

use crate::sns::report::{
    MainnetSns, MainnetSnsList, MainnetSnsNeuronPage, MainnetSnsNeurons, MainnetSnsProposal,
    MainnetSnsProposalPage, MainnetSnsProposals, MainnetSnsToken, SnsGovernanceParameters,
    SnsHostError, SnsListSource, SnsNeuronId, SnsNeuronsSource, SnsParamsSource, SnsProposalSource,
    SnsProposalTopicFilter, SnsProposalsSource, SnsSourceRequest, SnsTokenSource,
    live::fetch::{
        fetch_mainnet_sns_list, fetch_mainnet_sns_neuron_page, fetch_mainnet_sns_neurons,
        fetch_mainnet_sns_params, fetch_mainnet_sns_proposal, fetch_mainnet_sns_proposal_page,
        fetch_mainnet_sns_proposals, fetch_mainnet_sns_token,
    },
};

///
/// LiveSnsSource
///
/// Live mainnet SNS source used by report builders outside tests.
///

pub struct LiveSnsSource;

impl SnsListSource for LiveSnsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        fetch_mainnet_sns_list(request)
    }
}

impl SnsTokenSource for LiveSnsSource {
    fn fetch_sns_token(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError> {
        fetch_mainnet_sns_token(request, sns)
    }
}

impl SnsParamsSource for LiveSnsSource {
    fn fetch_sns_params(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError> {
        fetch_mainnet_sns_params(request, sns)
    }
}

impl SnsProposalSource for LiveSnsSource {
    fn fetch_sns_proposal(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        proposal_id: u64,
    ) -> Result<MainnetSnsProposal, SnsHostError> {
        fetch_mainnet_sns_proposal(request, sns, proposal_id)
    }
}

impl SnsProposalsSource for LiveSnsSource {
    fn fetch_sns_proposals(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        topic: SnsProposalTopicFilter,
    ) -> Result<MainnetSnsProposals, SnsHostError> {
        fetch_mainnet_sns_proposals(
            request,
            sns,
            limit,
            before_proposal_id,
            include_status,
            topic,
        )
    }

    fn fetch_sns_proposal_page(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> Result<MainnetSnsProposalPage, SnsHostError> {
        fetch_mainnet_sns_proposal_page(request, sns, limit, before_proposal_id)
    }
}

impl SnsNeuronsSource for LiveSnsSource {
    fn fetch_sns_neurons(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        fetch_mainnet_sns_neurons(request, sns, limit, owner_principal_id)
    }

    fn fetch_sns_neuron_page(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        fetch_mainnet_sns_neuron_page(request, sns, limit, start_page_at, owner_principal_id)
    }
}
