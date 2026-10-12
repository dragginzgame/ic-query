//! Module: sns::report::live::fetch::params
//!
//! Responsibility: fetch SNS governance parameters.
//! Does not own: lookup resolution, report assembly, cache IO, or rendering.
//! Boundary: queries one resolved SNS governance canister for parameters.

use super::governance_canister;
use crate::sns::report::{
    SnsCanisterMethod, SnsGovernanceParameters, SnsHostError,
    live::{
        convert::sns_governance_parameters,
        query::{query_canister, sns_agent},
        types::SnsGovernanceParametersWire,
    },
    source::{MainnetSns, SnsSourceRequest},
};

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_params(
    source: &crate::sns::LiveSnsSource,
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let governance_canister = governance_canister(sns)?;
    let method = SnsCanisterMethod::GetNervousSystemParameters.as_str();
    let parameters: SnsGovernanceParametersWire = query_canister(
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
