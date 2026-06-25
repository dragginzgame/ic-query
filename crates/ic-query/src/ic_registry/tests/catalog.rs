use super::{fixtures::*, *};

#[test]
fn registry_records_convert_to_catalog_domain_structs() {
    let request = registry_fetch_request();
    let subnet_records = BTreeMap::from([
        (
            SUBNET_A.to_string(),
            subnet_record(SubnetType::Application, 34),
        ),
        (SUBNET_B.to_string(), subnet_record(SubnetType::System, 13)),
    ]);
    let catalog = catalog_from_parts_for_test(
        &request,
        42,
        subnet_list_record([SUBNET_A, SUBNET_B]),
        routing_table_record([(CANISTER_A, CANISTER_A, SUBNET_A)]),
        subnet_records,
    )
    .expect("catalog");

    assert_eq!(catalog.registry_version, 42);
    assert_eq!(catalog.subnets.len(), 2);
    assert_eq!(catalog.routing_ranges.len(), 1);
    let fiduciary = catalog.subnet_by_principal(SUBNET_A).expect("fiduciary");
    assert_eq!(
        fiduciary.subnet_specialization,
        SubnetSpecialization::Fiduciary
    );
    assert_eq!(fiduciary.node_count, Some(34));
    assert!(fiduciary.charges_apply_by_default);
    let system = catalog.subnet_by_principal(SUBNET_B).expect("system");
    assert_eq!(system.subnet_kind, SubnetKind::System);
    assert!(!system.charges_apply_by_default);
}

#[test]
fn registry_records_preserve_cloud_engine_subnet_type() {
    let request = registry_fetch_request();
    let subnet_records = BTreeMap::from([(
        SUBNET_A.to_string(),
        subnet_record(SubnetType::CloudEngine, 13),
    )]);
    let catalog = catalog_from_parts_for_test(
        &request,
        42,
        subnet_list_record([SUBNET_A]),
        routing_table_record([(CANISTER_A, CANISTER_A, SUBNET_A)]),
        subnet_records,
    )
    .expect("catalog");

    let subnet = catalog.subnet_by_principal(SUBNET_A).expect("subnet");
    assert_eq!(subnet.subnet_kind, SubnetKind::CloudEngine);
    assert!(subnet.charges_apply_by_default);
}
