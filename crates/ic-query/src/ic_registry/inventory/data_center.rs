use super::{INVENTORY_FETCH_CONCURRENCY, keys::data_center_record_key};
use crate::ic_registry::{
    RegistryFetchError, normalized_data_center_id, transport::get_registry_record,
};
use futures_util::{StreamExt, TryStreamExt, stream};
use ic_host_tools::registry::proto::{DataCenterRecord, NodeOperatorRecord};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::Principal;
use std::collections::{BTreeMap, BTreeSet};

pub(super) async fn fetch_data_center_records_for_inventory(
    agent: &Agent,
    registry_canister: &Principal,
    registry_version: u64,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, DataCenterRecord>, RegistryFetchError> {
    let data_center_ids = node_operator_records
        .values()
        .filter_map(|record| normalized_data_center_id(&record.dc_id))
        .collect::<BTreeSet<_>>();
    stream::iter(data_center_ids)
        .map(|data_center_id| async move {
            let key = data_center_record_key(&data_center_id);
            let record = get_registry_record::<DataCenterRecord>(
                agent,
                registry_canister,
                &key,
                registry_version,
                "DataCenterRecord",
            )
            .await?;
            Ok::<_, RegistryFetchError>((data_center_id, record))
        })
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await
}
