use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::{
    ic_registry::{
        CANISTER_RANGES_KEY_PREFIX, MainnetRegistryFetchRequest, RegistryFetchError,
        SubnetCatalogRegistryFailure,
        catalog::{
            CatalogReadContext, CatalogRegistryReader, catalog_from_registry_records_detailed,
        },
        transport::{
            RegistryQueryCounter, get_latest_version_counted, get_registry_key_family_counted,
            get_registry_versioned_value_counted,
        },
        wire::{RegistryVersionedValue, RegistryVersionedValueFailure},
    },
    subnet_catalog::{
        RawSubnetCatalog, SubnetCatalogField, SubnetCatalogRegistryRecordKind,
        SubnetCatalogRegistryRecordSubject, SubnetCatalogRoutingSource, SubnetCatalogSubject,
    },
};
use ic_host_tools::registry::proto::{RoutingTable, SubnetListRecord};
use ic_host_tools::registry::routing_shards::{
    canister_range_start_from_key, validate_routing_table_shard_bounds,
};
use ic_platform::candid::contract::Principal;

/// Fetch one exact-version mainnet Subnet Catalog on the caller's async runtime.
pub async fn fetch_mainnet_subnet_catalog_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<RawSubnetCatalog, RegistryFetchError> {
    fetch_mainnet_subnet_catalog_detailed_async(request)
        .await
        .map_err(|failure| failure.source)
}

pub(in crate::ic_registry) async fn fetch_mainnet_subnet_catalog_detailed_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<RawSubnetCatalog, Box<SubnetCatalogRegistryFailure>> {
    fetch_with_acquisition(request, std::sync::Arc::default()).await
}

pub async fn fetch_with_acquisition(
    request: &MainnetRegistryFetchRequest,
    acquisition: std::sync::Arc<crate::ic_registry::RegistryAcquisition>,
) -> Result<RawSubnetCatalog, Box<SubnetCatalogRegistryFailure>> {
    let query_counter =
        RegistryQueryCounter::with_acquisition(request.endpoint.clone(), acquisition);
    query_counter.emit(crate::ic_registry::SubnetCatalogProgressPhase::EndpointStarted);
    let agent = mainnet_agent(request).map_err(|source| {
        SubnetCatalogRegistryFailure::new(
            None,
            Some(SubnetCatalogSubject::Endpoint(request.endpoint.clone())),
            source,
        )
    })?;
    let registry_canister = mainnet_registry_canister().map_err(|source| {
        SubnetCatalogRegistryFailure::new(
            None,
            Some(SubnetCatalogSubject::Field(
                SubnetCatalogField::RegistryCanister,
            )),
            source,
        )
    })?;
    let registry_version = get_latest_version_counted(&agent, &registry_canister, &query_counter)
        .await
        .map_err(latest_version_failure)?;
    query_counter.emit(crate::ic_registry::SubnetCatalogProgressPhase::Pinned { registry_version });
    let reader = AgentCatalogRegistryReader {
        agent: &agent,
        registry_canister: &registry_canister,
        query_counter: &query_counter,
    };
    collect_pinned_catalog(request, registry_version, &reader).await
}

async fn collect_pinned_catalog<R>(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    reader: &R,
) -> Result<RawSubnetCatalog, Box<SubnetCatalogRegistryFailure>>
where
    R: CatalogRegistryReader,
{
    let mut context = CatalogReadContext::new(request, registry_version, reader);

    let subnet_list_subject = SubnetCatalogRegistryRecordSubject::subnet_list();
    let (subnet_list, _) = context
        .record::<SubnetListRecord>(&subnet_list_subject)
        .await?;

    let shard_keys = reader
        .key_family(CANISTER_RANGES_KEY_PREFIX, registry_version)
        .await
        .map_err(|source| {
            context.record_failure(
                SubnetCatalogRegistryRecordSubject::keyed(
                    SubnetCatalogRegistryRecordKind::RoutingTable,
                    format!("{CANISTER_RANGES_KEY_PREFIX}*"),
                ),
                None,
                source,
            )
        })?;

    if shard_keys.is_empty() {
        return Err(context.record_failure(
            SubnetCatalogRegistryRecordSubject::keyed(
                SubnetCatalogRegistryRecordKind::RoutingTable,
                format!("{CANISTER_RANGES_KEY_PREFIX}*"),
            ),
            None,
            RegistryFetchError::InvalidRegistryKeyFamily {
                reason: format!(
                    "pinned mainnet Registry version {registry_version} has no authoritative canister-ranges shards"
                ),
            },
        ));
    }

    let mut shards = Vec::with_capacity(shard_keys.len());
    for key in shard_keys {
        let range_start =
            canister_range_start_from_key(CANISTER_RANGES_KEY_PREFIX, &key).map_err(|reason| {
                context.record_failure(
                    SubnetCatalogRegistryRecordSubject::keyed(
                        SubnetCatalogRegistryRecordKind::RoutingTable,
                        &key,
                    ),
                    None,
                    RegistryFetchError::InvalidRegistryKeyFamily { reason },
                )
            })?;
        shards.push((
            SubnetCatalogRegistryRecordSubject::canister_ranges(range_start.into()),
            range_start,
        ));
    }
    let mut routing_table = RoutingTable::default();
    for (index, (subject, range_start)) in shards.iter().enumerate() {
        let (shard, returned_version) = context.record::<RoutingTable>(subject).await?;
        validate_routing_table_shard_bounds(
            &subject.key,
            range_start,
            &shard,
            shards.get(index + 1).map(|(_, next_start)| next_start),
        )
        .map_err(|reason| {
            context.record_failure(
                subject.clone(),
                Some(returned_version),
                RegistryFetchError::InvalidRegistryKeyFamily { reason },
            )
        })?;
        routing_table.entries.extend(shard.entries);
    }
    let routing_source = SubnetCatalogRoutingSource::CanisterRanges;

    catalog_from_registry_records_detailed(context, subnet_list, routing_table, routing_source)
        .await
}

struct AgentCatalogRegistryReader<'a> {
    agent: &'a ic_platform::agent::Agent,
    registry_canister: &'a Principal,
    query_counter: &'a RegistryQueryCounter,
}

impl CatalogRegistryReader for AgentCatalogRegistryReader<'_> {
    async fn key_family(
        &self,
        prefix: &str,
        registry_version: u64,
    ) -> Result<Vec<String>, RegistryFetchError> {
        get_registry_key_family_counted(
            self.agent,
            self.registry_canister,
            prefix,
            registry_version,
            self.query_counter,
        )
        .await
    }

    async fn value(
        &self,
        key: &str,
        registry_version: u64,
    ) -> Result<RegistryVersionedValue, RegistryVersionedValueFailure> {
        get_registry_versioned_value_counted(
            self.agent,
            self.registry_canister,
            key,
            registry_version,
            self.query_counter,
        )
        .await
    }

    fn query_call_count(&self) -> u64 {
        self.query_counter.call_count()
    }
}

fn latest_version_failure(source: RegistryFetchError) -> Box<SubnetCatalogRegistryFailure> {
    SubnetCatalogRegistryFailure::new(
        None,
        Some(SubnetCatalogSubject::RegistryLatestVersion),
        source,
    )
}

#[cfg(test)]
mod tests;
