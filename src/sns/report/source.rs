use super::{
    SnsGovernanceParameters, SnsHostError, SnsNeuronRow, SnsProposalRow, SnsTokenMetadataRow,
    SnsTokenStandardRow,
};
use candid::{CandidType, Deserialize};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsFetchRequest {
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MainnetSnsList {
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub sns_instances: Vec<MainnetSns>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MainnetSns {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MainnetSnsCanisters {
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MainnetSnsToken {
    pub token_name: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub transfer_fee: String,
    pub total_supply: String,
    pub minting_account_owner: Option<String>,
    pub minting_account_subaccount_hex: Option<String>,
    pub ledger_index_canister_id: Option<String>,
    pub ledger_index_error: Option<String>,
    pub supported_standards: Vec<SnsTokenStandardRow>,
    pub metadata: Vec<SnsTokenMetadataRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MainnetSnsNeurons {
    pub neurons: Vec<SnsNeuronRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MainnetSnsProposals {
    pub proposals: Vec<SnsProposalRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MainnetSnsProposal {
    pub proposal: SnsProposalRow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MainnetSnsNeuronPage {
    pub neurons: Vec<SnsNeuronRow>,
    pub last_cursor: Option<SnsNeuronId>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SnsNeuronId {
    pub id: Vec<u8>,
}

pub(super) trait SnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError>;
}

pub(super) trait SnsTokenSource: SnsListSource {
    fn fetch_sns_token(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError>;
}

pub(super) trait SnsParamsSource: SnsListSource {
    fn fetch_sns_params(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError>;
}

pub(super) trait SnsProposalSource: SnsListSource {
    fn fetch_sns_proposal(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        proposal_id: u64,
    ) -> Result<MainnetSnsProposal, SnsHostError>;
}

pub(super) trait SnsProposalsSource: SnsListSource {
    fn fetch_sns_proposals(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
    ) -> Result<MainnetSnsProposals, SnsHostError>;
}

pub(super) trait SnsNeuronsSource: SnsListSource {
    fn fetch_sns_neurons(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError>;

    fn fetch_sns_neuron_page(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError>;
}
