use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::{
    ic_registry::{
        MainnetRegistryFetchRequest, ROUTING_TABLE_KEY, RegistryFetchError, SUBNET_LIST_KEY,
        SubnetCatalogRegistryFailure,
        catalog::catalog_from_registry_records_detailed,
        proto::{RoutingTable, SubnetListRecord},
        transport::{
            RegistryQueryCounter, decode_message, get_latest_version_counted,
            get_registry_value_counted,
        },
    },
    subnet_catalog::{
        RawSubnetCatalog, SubnetCatalogField, SubnetCatalogRegistryRecordKind,
        SubnetCatalogRegistryRecordSubject, SubnetCatalogSubject,
    },
};

pub(in crate::ic_registry) async fn fetch_mainnet_subnet_catalog_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<RawSubnetCatalog, RegistryFetchError> {
    fetch_mainnet_subnet_catalog_detailed_async(request)
        .await
        .map_err(|failure| failure.source)
}

pub(in crate::ic_registry) async fn fetch_mainnet_subnet_catalog_detailed_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<RawSubnetCatalog, SubnetCatalogRegistryFailure> {
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
    let query_counter = RegistryQueryCounter::default();
    let registry_version = get_latest_version_counted(&agent, &registry_canister, &query_counter)
        .await
        .map_err(latest_version_failure)?;
    let (subnet_list_bytes, routing_table_bytes) = futures::try_join!(
        get_catalog_record(
            &agent,
            &registry_canister,
            SUBNET_LIST_KEY,
            SubnetCatalogRegistryRecordKind::SubnetList,
            registry_version,
            &query_counter,
        ),
        get_catalog_record(
            &agent,
            &registry_canister,
            ROUTING_TABLE_KEY,
            SubnetCatalogRegistryRecordKind::RoutingTable,
            registry_version,
            &query_counter,
        ),
    )?;
    let subnet_list = decode_message::<SubnetListRecord>("SubnetListRecord", &subnet_list_bytes)
        .map_err(|source| {
            record_failure(
                registry_version,
                SUBNET_LIST_KEY,
                SubnetCatalogRegistryRecordKind::SubnetList,
                source,
            )
        })?;
    let routing_table = decode_message::<RoutingTable>("RoutingTable", &routing_table_bytes)
        .map_err(|source| {
            record_failure(
                registry_version,
                ROUTING_TABLE_KEY,
                SubnetCatalogRegistryRecordKind::RoutingTable,
                source,
            )
        })?;
    catalog_from_registry_records_detailed(
        request,
        registry_version,
        &agent,
        &registry_canister,
        subnet_list,
        routing_table,
        &query_counter,
    )
    .await
}

const fn latest_version_failure(source: RegistryFetchError) -> SubnetCatalogRegistryFailure {
    SubnetCatalogRegistryFailure::new(
        None,
        Some(SubnetCatalogSubject::RegistryRecord(
            SubnetCatalogRegistryRecordSubject::latest_version(),
        )),
        source,
    )
}

async fn get_catalog_record(
    agent: &ic_agent::Agent,
    registry_canister: &candid::Principal,
    key: &str,
    kind: SubnetCatalogRegistryRecordKind,
    registry_version: u64,
    query_counter: &RegistryQueryCounter,
) -> Result<Vec<u8>, SubnetCatalogRegistryFailure> {
    get_registry_value_counted(
        agent,
        registry_canister,
        key,
        registry_version,
        query_counter,
    )
    .await
    .map_err(|source| record_failure(registry_version, key, kind, source))
}

fn record_failure(
    registry_version: u64,
    key: &str,
    kind: SubnetCatalogRegistryRecordKind,
    source: RegistryFetchError,
) -> SubnetCatalogRegistryFailure {
    SubnetCatalogRegistryFailure::new(
        Some(registry_version),
        Some(SubnetCatalogSubject::RegistryRecord(
            SubnetCatalogRegistryRecordSubject::keyed(kind, key),
        )),
        source,
    )
}

#[cfg(test)]
mod detailed_failure_tests {
    use super::*;

    #[test]
    fn latest_version_failure_does_not_fabricate_a_registry_version() {
        let failure = latest_version_failure(RegistryFetchError::ProtobufDecode {
            message: "RegistryGetLatestVersionResponse",
            reason: "fixture".to_string(),
        });

        assert_eq!(failure.registry_version, None);
        assert_eq!(
            failure.subject,
            Some(SubnetCatalogSubject::RegistryRecord(
                SubnetCatalogRegistryRecordSubject {
                    kind: SubnetCatalogRegistryRecordKind::LatestVersion,
                    key: None,
                    subnet: None,
                }
            ))
        );
    }

    #[test]
    fn pinned_record_failures_retain_version_key_and_record_kind() {
        let failure = record_failure(
            771_992,
            SUBNET_LIST_KEY,
            SubnetCatalogRegistryRecordKind::SubnetList,
            RegistryFetchError::MissingValue {
                key: SUBNET_LIST_KEY.to_string(),
            },
        );

        assert_eq!(failure.registry_version, Some(771_992));
        assert_eq!(
            failure.subject,
            Some(SubnetCatalogSubject::RegistryRecord(
                SubnetCatalogRegistryRecordSubject {
                    kind: SubnetCatalogRegistryRecordKind::SubnetList,
                    key: Some(SUBNET_LIST_KEY.to_string()),
                    subnet: None,
                }
            ))
        );
    }
}
