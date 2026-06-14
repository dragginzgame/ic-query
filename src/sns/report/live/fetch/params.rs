use super::{block_on_sns, governance_canister};
use crate::sns::report::{
    SnsGovernanceParameters, SnsHostError,
    live::query::{query_canister, sns_agent},
    source::{MainnetSns, SnsFetchRequest},
};

pub(in crate::sns::report::live) fn fetch_mainnet_sns_params(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_params_async(request, sns))
}

async fn fetch_mainnet_sns_params_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister = governance_canister(sns)?;
    query_canister(
        &agent,
        &governance_canister,
        "get_nervous_system_parameters",
        "get_nervous_system_parameters",
        "SnsGovernanceParameters",
        &(),
    )
    .await
}
