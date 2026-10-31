//! Module: sns::report::live::fetch::params
//!
//! Responsibility: fetch SNS governance parameters.
//! Does not own: lookup resolution, report assembly, cache IO, or rendering.
//! Boundary: queries one resolved SNS governance canister for parameters.

use super::governance_canister;
use crate::candid_call::query_canister;
use crate::report::CollectionSourceRequest;
use crate::sns::report::{
    SnsCanisterMethod, SnsGovernanceParameters, SnsHostError,
    live::{
        convert::sns_governance_parameters, query::sns_agent, types::SnsGovernanceParametersWire,
    },
    source::MainnetSns,
};

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_params(
    source: &crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let governance_canister = governance_canister(sns)?;
    let method = SnsCanisterMethod::GetNervousSystemParameters.as_str();
    let parameters: SnsGovernanceParametersWire = query_canister::<_, _, SnsHostError>(
        agent,
        &governance_canister,
        method,
        method,
        "SnsGovernanceParameters",
        &(),
    )
    .await?;
    Ok(sns_governance_parameters(parameters))
}
