//! Module: sns::report::lookup::resolve
//!
//! Responsibility: resolve SNS lookup input into one deployed SNS.
//! Does not own: command parsing, live transport internals, or report assembly.
//! Boundary: resolves id/root against raw SNS-W inventory before targeted metadata enrichment.

use crate::sns::report::MainnetSnsCanisters;
use crate::sns::report::lookup::{model::SnsLookup, request::parse_sns_root_canister_input};
use crate::sns::report::{
    SnsHostError, SnsLookupRequest, enforce_mainnet_network,
    source::{
        SnsDiscoverySource, model::list::join_selected_mainnet_sns, validate_mainnet_sns_inventory,
    },
};

/// Resolve a user SNS lookup input to one deployed SNS and fetch context.
pub(in crate::sns::report) async fn resolve_sns_lookup(
    request: &SnsLookupRequest,
    source: &dyn SnsDiscoverySource,
) -> Result<SnsLookup, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetch_request = crate::report::CollectionSourceRequest::from_unix_secs(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query",
    );
    let mut inventory = source.fetch_sns_inventory(&fetch_request).await?;
    validate_mainnet_sns_inventory(&fetch_request, &inventory)?;
    let index = resolve_sns(&inventory.sns_instances, &request.input)?;
    let metadata = source
        .fetch_sns_metadata(&fetch_request, &inventory.sns_instances[index..=index])
        .await?;
    let target = inventory.sns_instances.swap_remove(index);
    let mut sns = join_selected_mainnet_sns(target, metadata)?;
    sns.id = index + 1;
    Ok(SnsLookup { fetch_request, sns })
}

fn resolve_sns(instances: &[MainnetSnsCanisters], input: &str) -> Result<usize, SnsHostError> {
    if let Ok(id) = input.parse::<usize>() {
        return id
            .checked_sub(1)
            .filter(|index| *index < instances.len())
            .ok_or(SnsHostError::UnknownSnsId {
                id,
                sns_count: instances.len(),
            });
    }

    let root_canister_id = parse_sns_root_canister_input(input)?;
    instances
        .iter()
        .position(|sns| sns.root_canister_id == root_canister_id)
        .ok_or(SnsHostError::UnknownSnsRoot { root_canister_id })
}
