use super::*;

#[test]
fn topology_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("topology"),
        OsString::from("summary"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology summary"));
    assert!(message.contains("icq --network ic nns topology coverage"));
    assert!(message.contains("icq --network ic nns topology versions"));
    assert!(message.contains("icq --network ic nns topology health"));
    assert!(message.contains("icq --network ic nns topology gaps"));
    assert!(message.contains("icq --network ic nns topology capacity"));
    assert!(message.contains("icq --network ic nns topology regions"));
    assert!(message.contains("icq --network ic nns topology providers"));
}

#[test]
fn topology_coverage_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("topology"),
        OsString::from("coverage"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology coverage"));
}

#[test]
fn topology_versions_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("topology"),
        OsString::from("versions"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology versions"));
}

#[test]
fn topology_health_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("topology"),
        OsString::from("health"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology health"));
}

#[test]
fn topology_gaps_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("topology"),
        OsString::from("gaps"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology gaps"));
}

#[test]
fn topology_capacity_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("topology"),
        OsString::from("capacity"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology capacity"));
}

#[test]
fn topology_regions_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("topology"),
        OsString::from("regions"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology regions"));
}

#[test]
fn topology_providers_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("topology"),
        OsString::from("providers"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology providers"));
}

#[test]
fn topology_refresh_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("topology"),
        OsString::from("refresh"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology refresh"));
}
