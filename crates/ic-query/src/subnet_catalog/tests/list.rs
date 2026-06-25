use super::{fixtures::*, *};

#[test]
fn list_report_loads_cached_catalog_and_caps_ranges() {
    let root = temp_dir("ic-query-subnet-list");
    write_catalog(&root, fixture_catalog());
    let request = list_request(&root);

    let report = build_subnet_catalog_list_report(&request).expect("list report");
    let text = subnet_catalog_list_report_text(&report);

    let _ = fs::remove_dir_all(root);
    assert_eq!(report.subnets.len(), 2);
    assert_eq!(report.subnets[0].range_count, 2);
    assert_eq!(report.subnets[0].ranges_shown, 1);
    assert!(text.contains("SUBNET"));
    assert!(text.contains("SPEC"));
    assert!(!text.contains("SPECIALIZATION"));
    for subnet in &report.subnets {
        assert!(text.contains(&compact_principal(&subnet.subnet_principal)));
        assert!(!text.contains(&subnet.subnet_principal));
    }
    assert!(!text.contains("FETCHED_AT"));
    assert!(text.contains("showing 1 of 2 ranges"));
}

#[test]
fn list_report_refreshes_missing_catalog() {
    let root = temp_dir("ic-query-subnet-list-refresh");
    let mut catalog = fixture_catalog();
    catalog.registry_version = 987_654;
    let source = FixtureRefreshSource::ok(catalog);
    let request = list_request(&root);

    let report =
        build_subnet_catalog_list_report_with_source(&request, &source).expect("list report");
    let cached = load_cached_subnet_catalog(&cache_request(&root)).expect("cached catalog");

    let _ = fs::remove_dir_all(root);
    assert_eq!(report.registry_version, 987_654);
    assert_eq!(cached.catalog.registry_version, 987_654);
}

#[test]
fn list_report_verbose_text_keeps_full_metadata() {
    let root = temp_dir("ic-query-subnet-list-verbose");
    write_catalog(&root, fixture_catalog());
    let request = list_request(&root);

    let report = build_subnet_catalog_list_report(&request).expect("list report");
    let text = subnet_catalog_list_report_verbose_text(&report);

    let _ = fs::remove_dir_all(root);
    assert!(text.contains("catalog_path:"));
    assert!(text.contains("SPECIALIZATION"));
    assert!(text.contains("FETCHED_AT"));
    assert!(text.contains(SUBNET_A));
}
