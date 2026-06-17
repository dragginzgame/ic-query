//! Module: sns::report::lookup::resolve
//!
//! Responsibility: resolve SNS lookup input into one deployed SNS.
//! Does not own: command parsing, live transport internals, or report assembly.
//! Boundary: fetches the SNS list through a source and resolves id/root input.

use crate::sns::report::lookup::{
    model::SnsLookup,
    network::enforce_mainnet_network,
    request::fetch_request_from_parts,
    sort::{assign_sns_ids_in_current_order, sort_mainnet_sns_instances},
};
use crate::sns::report::{
    SnsHostError, SnsListSort, SnsLookupRequest,
    source::{MainnetSns, SnsListSource},
};
use candid::Principal;

/// Resolve a user SNS lookup input to one deployed SNS and fetch context.
pub(in crate::sns::report) fn resolve_sns_lookup(
    request: &SnsLookupRequest,
    source: &dyn SnsListSource,
) -> Result<SnsLookup, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetch_request = fetch_request_from_parts(
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    );
    let mut list = source.fetch_deployed_snses(&fetch_request)?;
    assign_sns_ids_in_current_order(&mut list.sns_instances);
    sort_mainnet_sns_instances(&mut list.sns_instances, SnsListSort::Id);
    let (id, sns) = resolve_sns(&list.sns_instances, &request.input)?;
    Ok(SnsLookup {
        fetch_request,
        list,
        id,
        sns,
    })
}

fn resolve_sns(instances: &[MainnetSns], input: &str) -> Result<(usize, MainnetSns), SnsHostError> {
    if let Ok(id) = input.parse::<usize>() {
        return instances
            .iter()
            .find(|sns| sns.id == id)
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
        .find(|sns| sns.root_canister_id == root_canister_id)
        .map(|sns| (sns.id, sns.clone()))
        .ok_or(SnsHostError::UnknownSnsRoot { root_canister_id })
}
