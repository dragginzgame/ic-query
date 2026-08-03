mod canisters;
mod metrics;
mod neurons;
mod params;
mod proposals;
mod requests;
mod reward;
mod sns;
mod swap;
mod token;
mod upgrade;

pub(in crate::sns::report::tests) use canisters::FixtureSnsCanisterSource;
pub(in crate::sns::report::tests) use metrics::{
    FixtureSnsMetricsSource, MutatingFixtureSnsMetricsSource, NoCallSnsMetricsSource,
};
pub(in crate::sns::report::tests) use neurons::{
    FixtureSnsNeuronSource, FixtureSnsNeuronsSource, NEURON_A, NoLiveSnsNeuronsSource,
    PagedFixtureSnsNeuronsSource, fixture_sns_neuron,
};
pub(in crate::sns::report::tests) use params::{
    FixtureSnsParamsSource, fixture_sns_governance_parameters,
};
pub(in crate::sns::report::tests) use proposals::{
    FixtureSnsProposalSource, FixtureSnsProposalsSource, NoLiveSnsProposalsSource,
    fixture_proposal_row,
};
pub(in crate::sns::report::tests) use requests::{
    info_request, list_request, metrics_request, neuron_request, neurons_request, params_request,
    proposal_request, proposals_request, reward_checkpoint_request, sns_neurons_refresh_request,
    sns_proposals_refresh_request, swap_request, token_request, upgrade_request,
};
pub(in crate::sns::report::tests) use reward::{
    FixtureSnsRewardSource, fixture_reward_page, fixture_reward_row,
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
