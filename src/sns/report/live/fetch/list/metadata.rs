use crate::sns::report::live::{
    convert::{mainnet_sns_from_canisters_and_metadata, metadata_error_summary},
    query::principal_from_text,
    query::query_canister,
    types::{GetMetadataRequest, GetMetadataResponse},
};
use crate::sns::report::{
    SNS_METADATA_CONCURRENCY, SnsHostError,
    source::{MainnetSns, MainnetSnsCanisters},
};
use candid::Principal;
use futures::{StreamExt, stream};
use ic_agent::Agent;

pub(super) async fn fetch_mainnet_sns_metadata_rows(
    agent: &Agent,
    sns_canisters: Vec<MainnetSnsCanisters>,
) -> Result<Vec<MainnetSns>, SnsHostError> {
    let fetched = stream::iter(
        sns_canisters
            .into_iter()
            .map(|sns| fetch_mainnet_sns_metadata(agent, sns)),
    )
    .buffered(SNS_METADATA_CONCURRENCY)
    .collect::<Vec<_>>()
    .await;
    let mut sns_instances = Vec::with_capacity(fetched.len());
    for sns in fetched {
        sns_instances.push(sns?);
    }
    Ok(sns_instances)
}

async fn fetch_mainnet_sns_metadata(
    agent: &Agent,
    sns: MainnetSnsCanisters,
) -> Result<MainnetSns, SnsHostError> {
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let (metadata, metadata_error) =
        match fetch_governance_metadata(agent, &governance_canister).await {
            Ok(metadata) => (metadata, None),
            Err(err) => match metadata_error_summary(&err) {
                Some(summary) => (GetMetadataResponse::default(), Some(summary)),
                None => return Err(err),
            },
        };
    Ok(mainnet_sns_from_canisters_and_metadata(
        sns,
        metadata,
        metadata_error,
    ))
}

async fn fetch_governance_metadata(
    agent: &Agent,
    governance_canister: &Principal,
) -> Result<GetMetadataResponse, SnsHostError> {
    query_canister(
        agent,
        governance_canister,
        "get_metadata",
        "GetMetadataRequest",
        "GetMetadataResponse",
        &GetMetadataRequest {},
    )
    .await
}
