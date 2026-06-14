use super::{
    MainnetSns, MainnetSnsList, SnsFetchRequest, SnsHostError, SnsListRequest, SnsListSort,
    SnsListSource, SnsLookupRequest,
};
use crate::subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs};
use candid::Principal;

pub(super) fn resolve_sns_lookup(
    request: &SnsLookupRequest,
    source: &dyn SnsListSource,
) -> Result<(SnsFetchRequest, MainnetSnsList, usize, MainnetSns), SnsHostError> {
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
    Ok((fetch_request, list, id, sns))
}

pub(super) fn lookup_request_from_parts(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    input: &str,
) -> SnsLookupRequest {
    SnsLookupRequest {
        network: network.to_string(),
        source_endpoint: source_endpoint.to_string(),
        now_unix_secs,
        input: input.to_string(),
    }
}

pub(super) fn sns_list_fetch_request(
    request: &SnsListRequest,
) -> Result<SnsFetchRequest, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    Ok(fetch_request_from_parts(
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    ))
}

pub(super) fn assign_sns_ids_in_current_order(instances: &mut [MainnetSns]) {
    for (index, sns) in instances.iter_mut().enumerate() {
        sns.id = index + 1;
    }
}

pub(super) fn sort_mainnet_sns_instances(instances: &mut [MainnetSns], sort: SnsListSort) {
    match sort {
        SnsListSort::Id => sort_mainnet_sns_instances_by_id(instances),
        SnsListSort::Name => instances.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.id.cmp(&right.id))
        }),
    }
}

pub(super) fn enforce_mainnet_network(network: &str) -> Result<(), SnsHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(SnsHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}

fn sort_mainnet_sns_instances_by_id(instances: &mut [MainnetSns]) {
    instances.sort_by_key(|sns| sns.id);
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

fn fetch_request_from_parts(
    source_endpoint: &str,
    now_unix_secs: u64,
    fetched_by: String,
) -> SnsFetchRequest {
    SnsFetchRequest {
        endpoint: source_endpoint.to_string(),
        fetched_at: format_utc_timestamp_secs(now_unix_secs),
        fetched_by,
    }
}
