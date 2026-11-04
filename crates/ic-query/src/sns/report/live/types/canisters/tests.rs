use super::GetSnsCanistersSummaryRequest;

#[test]
fn health_request_explicitly_disables_inventory_updates() {
    assert_eq!(
        GetSnsCanistersSummaryRequest::read_only().update_canister_list,
        Some(false)
    );
}
