use ic_platform::nns_registry_types::GetSubnetForCanisterResponseOk;

use super::{
    PlacementError, Principal, REPLY_LIMITS, candid_codec, decode_subnet, decoder, report,
};

fn reply(subnet_id: Option<Principal>) -> Vec<u8> {
    let response: Result<_, String> = Ok(GetSubnetForCanisterResponseOk { subnet_id });
    candid_codec::encode_one(&response, 1024).expect("fixture encodes")
}

#[test]
fn subnet_identity_survives_decoding_and_json_keeps_raw_evidence() {
    let canister = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    let subnet = Principal::from_slice(&[17; 29]);
    let decoded = decode_subnet(&reply(Some(subnet))).unwrap();
    assert_eq!(decoded, subnet);
    let value = serde_json::to_value(report(canister, decoded, 9_007_199_254_740_993)).unwrap();
    assert_eq!(value["schema"], 1);
    assert_eq!(value["canister_id"], canister.to_text());
    assert_eq!(value["subnet_id"], subnet.to_text());
    let evidence = &value["evidence"];
    assert_eq!(
        evidence["observed_at_unix_seconds"].as_u64(),
        Some(9_007_199_254_740_993)
    );
    assert_eq!(evidence["source_method"], "get_subnet_for_canister");
    assert_eq!(evidence["endpoint"], "https://ic0.app");
    assert_eq!(evidence["source_canister"], "rwlgt-iiaaa-aaaaa-aaaaa-cai");
    assert_eq!(evidence["query_signature_verified"], true);
    assert_eq!(evidence["result_certified"], false);
    assert!(evidence["registry_version"].is_null());
    assert_eq!(evidence["selected_lookup_complete"], true);
}

#[test]
fn registry_rejection_and_missing_subnet_are_distinct_failures() {
    assert!(matches!(
        decode_subnet(&reply(None)),
        Err(PlacementError::MissingSubnet)
    ));
    let response: Result<GetSubnetForCanisterResponseOk, _> = Err("unmapped 🌍".to_owned());
    let bytes = candid_codec::encode_one(&response, 1024).unwrap();
    assert!(
        matches!(decode_subnet(&bytes), Err(PlacementError::RegistryRejected(message)) if message == "unmapped 🌍")
    );
}

#[test]
fn malformed_trailing_surplus_and_oversized_replies_are_refused() {
    assert!(matches!(
        decode_subnet(b"invalid"),
        Err(PlacementError::Decode(_))
    ));
    let mut trailing = reply(Some(Principal::from_slice(&[1])));
    trailing.push(0);
    assert!(matches!(
        decode_subnet(&trailing),
        Err(PlacementError::Decode(decoder::Error::Trailing))
    ));
    let response: Result<_, String> = Ok(GetSubnetForCanisterResponseOk { subnet_id: None });
    let surplus = candid_codec::arguments::encode_args(&(response, 1u8), 1024).unwrap();
    assert!(matches!(
        decode_subnet(&surplus),
        Err(PlacementError::Decode(decoder::Error::Arity))
    ));
    assert!(matches!(
        decode_subnet(&vec![0; REPLY_LIMITS.bytes + 1]),
        Err(PlacementError::Decode(decoder::Error::Limit))
    ));
}
