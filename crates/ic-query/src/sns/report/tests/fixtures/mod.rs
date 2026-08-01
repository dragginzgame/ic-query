mod canisters;
mod neurons;
mod params;
mod proposals;
mod requests;
mod sns;
mod swap;
mod token;
mod upgrade;

pub(in crate::sns::report::tests) use canisters::FixtureSnsCanisterSource;
pub(in crate::sns::report::tests) use neurons::{
    FixtureSnsNeuronsSource, NoLiveSnsNeuronsSource, PagedFixtureSnsNeuronsSource,
};
pub(in crate::sns::report::tests) use params::FixtureSnsParamsSource;
pub(in crate::sns::report::tests) use proposals::{
    FixtureSnsProposalSource, FixtureSnsProposalsSource, NoLiveSnsProposalsSource,
    fixture_proposal_row,
};
pub(in crate::sns::report::tests) use requests::{
    info_request, list_request, neurons_request, params_request, proposal_request,
    proposals_request, sns_neurons_refresh_request, sns_proposals_refresh_request, swap_request,
    token_request, upgrade_request,
};
pub(in crate::sns::report::tests) use sns::{
    FixtureSnsDiscoverySource, GOVERNANCE_A, INDEX_A, LEDGER_A,
    MetadataErrorFixtureSnsDiscoverySource, ROOT_A, SWAP_A, UnsortedFixtureSnsDiscoverySource,
    fixture_canisters_a, fixture_sns_a,
};
pub(in crate::sns::report::tests) use swap::{
    FixtureSnsSwapSource, MutatingFixtureSnsSwapSource, PartialFixtureSnsSwapSource,
    WrongTargetFixtureSnsSwapSource,
};
pub(in crate::sns::report::tests) use token::FixtureSnsTokenSource;
pub(in crate::sns::report::tests) use upgrade::{
    FixtureSnsUpgradeSource, MutatingFixtureSnsUpgradeSource, fixture_sns_version,
};
