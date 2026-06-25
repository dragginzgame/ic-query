use super::{fixtures::*, *};

#[test]
fn info_report_resolves_canister_and_marks_application_chargeable() {
    let root = temp_dir("ic-query-subnet-info");
    write_catalog(&root, fixture_catalog());
    let request = info_request(&root, CANISTER_A);

    let report = build_subnet_catalog_info_report(&request).expect("info report");

    let _ = fs::remove_dir_all(root);
    assert_eq!(report.resolved_as, "canister");
    assert_eq!(report.subnet_principal, SUBNET_A);
    assert!(report.charges_apply_to_subject);
    assert_eq!(
        report.charge_applicability_reason,
        "charged_user_canister_subnet"
    );
    assert_eq!(report.cycles_per_billion_instructions, Some(2_615_384_616));
}

#[test]
fn info_report_resolves_unique_subnet_prefix() {
    let root = temp_dir("ic-query-subnet-info-subnet-prefix");
    write_catalog(&root, fixture_catalog());
    let request = info_request(&root, "rwl");

    let report = build_subnet_catalog_info_report(&request).expect("info report");

    let _ = fs::remove_dir_all(root);
    assert_eq!(report.input_principal, "rwl");
    assert_eq!(report.resolved_as, "subnet");
    assert_eq!(report.resolved_from, "subnet_principal_prefix");
    assert_eq!(report.subnet_principal, SUBNET_A);
    assert_eq!(report.matched_canister_principal, None);
}

#[test]
fn info_report_rejects_canister_prefix() {
    let root = temp_dir("ic-query-subnet-info-canister-prefix");
    write_catalog(&root, fixture_catalog());
    let request = info_request(&root, "ryj");

    let err = build_subnet_catalog_info_report(&request).expect_err("canister prefix rejected");

    let _ = fs::remove_dir_all(root);
    assert!(matches!(
        err,
        SubnetCatalogHostError::Catalog(CatalogError::PrincipalPrefixNotFound { prefix })
            if prefix == "ryj"
    ));
}

#[test]
fn system_subnet_has_no_catalog_rate() {
    let root = temp_dir("ic-query-subnet-system");
    let mut catalog = fixture_catalog();
    catalog.subnets[0].subnet_kind = SubnetKind::System;
    catalog.subnets[0].charges_apply_by_default = false;
    write_catalog(&root, catalog);
    let request = info_request(&root, CANISTER_A);

    let report = build_subnet_catalog_info_report(&request).expect("info report");

    let _ = fs::remove_dir_all(root);
    assert!(!report.charges_apply_to_subject);
    assert_eq!(
        report.charge_applicability_reason,
        "system_subnet_unknown_subject"
    );
    assert_eq!(report.cycles_per_billion_instructions, None);
}

#[test]
fn cloud_engine_subnet_keeps_application_rate_class() {
    let root = temp_dir("ic-query-subnet-cloud-engine");
    let mut catalog = fixture_catalog();
    catalog.subnets[0].subnet_kind = SubnetKind::CloudEngine;
    catalog.subnets[0].subnet_label = "cloud_engine".to_string();
    catalog.subnets[0].charges_apply_by_default = true;
    write_catalog(&root, catalog);
    let request = info_request(&root, CANISTER_A);

    let report = build_subnet_catalog_info_report(&request).expect("info report");

    let _ = fs::remove_dir_all(root);
    assert_eq!(report.subnet_kind, SubnetKind::CloudEngine);
    assert!(report.charges_apply_to_subject);
    assert_eq!(
        report.charge_applicability_reason,
        "charged_user_canister_subnet"
    );
    assert_eq!(report.cycles_per_billion_instructions, Some(2_615_384_616));
}
