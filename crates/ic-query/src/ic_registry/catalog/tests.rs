use super::*;
use ic_platform::candid::contract::Principal;

const SUBNET: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";

#[test]
fn subnet_record_failure_retains_exact_pinned_subject() {
    let request = MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-08-20T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    };
    let subnet = Principal::from_text(SUBNET).expect("subnet");
    let key = crate::ic_registry::subnet_record_key(SUBNET);
    let failure = CatalogReadContext::new(&request, 882_110, &()).record_failure(
        SubnetCatalogRegistryRecordSubject::subnet_record(subnet),
        Some(882_000),
        RegistryFetchError::MissingValue { key: key.clone() },
    );

    assert_eq!(failure.evidence.registry_version, Some(882_110));
    assert_eq!(
        failure.evidence.subject,
        Some(SubnetCatalogSubject::RegistryRecord(
            SubnetCatalogRegistryRecordSubject {
                kind: SubnetCatalogRegistryRecordKind::SubnetRecord,
                key,
                subnet: Some(subnet),
                canister_range_start: None,
            }
        ))
    );
    assert_eq!(
        failure.evidence.returned_registry_value_version,
        Some(882_000)
    );
    assert_eq!(
        failure.evidence.source_endpoint.as_deref(),
        Some(request.endpoint.as_str())
    );
}

#[test]
fn routing_table_and_range_failures_retain_pinned_typed_subjects() {
    let request = MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-08-20T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    };
    let table_failure = CatalogReadContext::new(&request, 882_111, &()).record_failure(
        SubnetCatalogRegistryRecordSubject::legacy_routing_table(),
        None,
        RegistryFetchError::EmptyRoutingTable,
    );
    assert_eq!(table_failure.evidence.registry_version, Some(882_111));
    assert!(matches!(
        table_failure.evidence.subject,
        Some(SubnetCatalogSubject::RegistryRecord(
            SubnetCatalogRegistryRecordSubject {
                kind: SubnetCatalogRegistryRecordKind::RoutingTable,
                ..
            }
        ))
    ));

    let range_failure = routing_range_failure(
        7,
        SubnetCatalogField::RoutingRangeStart,
        RegistryFetchError::MissingField {
            field: "range.start",
        },
    );
    let mut range_failure = range_failure;
    range_failure.evidence.registry_version = Some(882_111);
    assert_eq!(range_failure.evidence.registry_version, Some(882_111));
    assert_eq!(
        range_failure.evidence.subject,
        Some(SubnetCatalogSubject::RegistryRoutingTableEntry {
            index: 7,
            field: Some(SubnetCatalogField::RoutingRangeStart),
        })
    );
}
