use super::super::{SnsNeuronRow, SnsProposalRow, SnsTokenMetadataRow, SnsTokenStandardRow};
use candid::{CandidType, Deserialize};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct SnsFetchRequest {
    pub(in crate::sns::report) endpoint: String,
    pub(in crate::sns::report) fetched_at: String,
    pub(in crate::sns::report) fetched_by: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsList {
    pub(in crate::sns::report) network: String,
    pub(in crate::sns::report) sns_wasm_canister_id: String,
    pub(in crate::sns::report) fetched_at: String,
    pub(in crate::sns::report) fetched_by: String,
    pub(in crate::sns::report) source_endpoint: String,
    pub(in crate::sns::report) sns_instances: Vec<MainnetSns>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSns {
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) name: String,
    pub(in crate::sns::report) description: Option<String>,
    pub(in crate::sns::report) url: Option<String>,
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
    pub(in crate::sns::report) ledger_canister_id: String,
    pub(in crate::sns::report) swap_canister_id: String,
    pub(in crate::sns::report) index_canister_id: String,
    pub(in crate::sns::report) metadata_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsCanisters {
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
    pub(in crate::sns::report) ledger_canister_id: String,
    pub(in crate::sns::report) swap_canister_id: String,
    pub(in crate::sns::report) index_canister_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsToken {
    pub(in crate::sns::report) token_name: String,
    pub(in crate::sns::report) token_symbol: String,
    pub(in crate::sns::report) decimals: u8,
    pub(in crate::sns::report) transfer_fee: String,
    pub(in crate::sns::report) total_supply: String,
    pub(in crate::sns::report) minting_account_owner: Option<String>,
    pub(in crate::sns::report) minting_account_subaccount_hex: Option<String>,
    pub(in crate::sns::report) ledger_index_canister_id: Option<String>,
    pub(in crate::sns::report) ledger_index_error: Option<String>,
    pub(in crate::sns::report) supported_standards: Vec<SnsTokenStandardRow>,
    pub(in crate::sns::report) metadata: Vec<SnsTokenMetadataRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsNeurons {
    pub(in crate::sns::report) neurons: Vec<SnsNeuronRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsProposals {
    pub(in crate::sns::report) proposals: Vec<SnsProposalRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsProposal {
    pub(in crate::sns::report) proposal: SnsProposalRow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsNeuronPage {
    pub(in crate::sns::report) neurons: Vec<SnsNeuronRow>,
    pub(in crate::sns::report) last_cursor: Option<SnsNeuronId>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report) struct SnsNeuronId {
    pub(in crate::sns::report) id: Vec<u8>,
}
