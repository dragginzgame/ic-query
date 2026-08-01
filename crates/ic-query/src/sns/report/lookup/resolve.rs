//! Module: sns::report::lookup::resolve
//!
//! Responsibility: resolve SNS lookup input into one deployed SNS.
//! Does not own: command parsing, live transport internals, or report assembly.
//! Boundary: resolves id/root against raw SNS-W inventory before targeted metadata enrichment.

use crate::sns::report::lookup::{model::SnsLookup, request::fetch_request_from_parts};
use crate::sns::report::{
    SnsHostError, SnsLookupRequest, enforce_mainnet_network,
    source::{
        MainnetSnsCanisters, MainnetSnsInventory, SnsDiscoverySource, join_mainnet_sns_inventory,
        validate_mainnet_sns_inventory,
    },
};
use candid::Principal;

/// Resolve a user SNS lookup input to one deployed SNS and fetch context.
pub(in crate::sns::report) fn resolve_sns_lookup(
    request: &SnsLookupRequest,
    source: &dyn SnsDiscoverySource,
) -> Result<SnsLookup, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetch_request = fetch_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    );
    let inventory = source.fetch_sns_inventory(&fetch_request)?;
    validate_mainnet_sns_inventory(&fetch_request, &inventory)?;
    let (id, target) = resolve_sns(&inventory.sns_instances, &request.input)?;
    let metadata = source.fetch_sns_metadata(&fetch_request, std::slice::from_ref(&target))?;
    let mut list = join_mainnet_sns_inventory(selected_inventory(inventory, target), metadata)?;
    let sns = list
        .sns_instances
        .first_mut()
        .ok_or_else(|| SnsHostError::InvalidSourceData {
            capability: "SNS metadata",
            reason: "metadata join returned no selected SNS".to_string(),
        })?;
    sns.id = id;
    let sns = sns.clone();
    Ok(SnsLookup {
        fetch_request,
        list,
        id,
        sns,
    })
}

fn resolve_sns(
    instances: &[MainnetSnsCanisters],
    input: &str,
) -> Result<(usize, MainnetSnsCanisters), SnsHostError> {
    if let Ok(id) = input.parse::<usize>() {
        return id
            .checked_sub(1)
            .and_then(|index| instances.get(index))
            .cloned()
            .map(|sns| (id, sns))
            .ok_or(SnsHostError::UnknownSnsId {
                id,
                sns_count: instances.len(),
            });
    }

    let root_canister_id = Principal::from_text(input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: input.to_string(),
        })?
        .to_text();
    instances
        .iter()
        .enumerate()
        .find(|(_, sns)| sns.root_canister_id == root_canister_id)
        .map(|(index, sns)| (index + 1, sns.clone()))
        .ok_or(SnsHostError::UnknownSnsRoot { root_canister_id })
}

fn selected_inventory(
    inventory: MainnetSnsInventory,
    target: MainnetSnsCanisters,
) -> MainnetSnsInventory {
    MainnetSnsInventory {
        network: inventory.network,
        sns_wasm_canister_id: inventory.sns_wasm_canister_id,
        fetched_at: inventory.fetched_at,
        fetched_by: inventory.fetched_by,
        source_endpoint: inventory.source_endpoint,
        sns_instances: vec![target],
    }
}
