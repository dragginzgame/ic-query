use super::*;

#[test]
fn selectors_retain_caller_text_and_parse_identity_once() {
    let numeric = SnsLookupRequest::new("ic", "https://icp-api.io", 0, "0002").unwrap();
    assert_eq!(numeric.input(), "0002");
    assert_eq!(numeric.id(), Some(2));
    assert_eq!(numeric.root_canister_id(), None);
    let root = "be2us-64aaa-aaaaa-qaabq-cai";
    let principal = SnsLookupRequest::new("ic", "https://icp-api.io", 0, root).unwrap();
    assert_eq!(principal.input(), root);
    assert_eq!(principal.id(), None);
    assert_eq!(principal.root_canister_id().unwrap().to_text(), root);
    let error =
        SnsLookupRequest::new("ic", "https://icp-api.io", 0, "not-a-principal").unwrap_err();
    assert_eq!(error.input, "not-a-principal");
}
