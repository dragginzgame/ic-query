use super::*;
use crate::certification::verify_canister_tree;
use ic_platform::certification::{Certificate, labeled as label, leaf};

#[test]
fn official_candid_shape_round_trips() {
    let response = response(Vec::new(), Vec::new());
    let bytes =
        ic_platform::candid::contract::encode_one(&response).expect("encode official CMC response");
    let decoded: IcpXdrConversionRateResponse =
        crate::candid_decode::decode_reply(&bytes).expect("decode official CMC response");

    assert_eq!(decoded, response);
}

#[test]
fn accepts_a_rate_leaf_committed_by_certified_data() {
    let canister = cmc_canister();
    let rate = rate();
    let encoded_rate = ic_platform::candid::contract::encode_one(&rate).expect("encode CMC rate");
    let hash_tree = label(ICP_XDR_CONVERSION_RATE_LABEL.to_vec(), leaf(encoded_rate));
    let certificate = certificate_for_tree(&canister, hash_tree.digest().to_vec());
    verify_canister_tree(&certificate, &canister, &hash_tree, "canister")
        .expect("matching certified-data witness");
    verify_rate_leaf(&hash_tree, &rate).expect("matching certified CMC rate leaf");
}

#[test]
fn rejects_a_leaf_for_a_different_rate() {
    let rate = rate();
    let different_rate = IcpXdrConversionRate {
        timestamp_seconds: rate.timestamp_seconds,
        xdr_permyriad_per_icp: rate.xdr_permyriad_per_icp + 1,
    };
    let encoded_rate =
        ic_platform::candid::contract::encode_one(&different_rate).expect("encode CMC rate");
    let hash_tree = label(ICP_XDR_CONVERSION_RATE_LABEL.to_vec(), leaf(encoded_rate));

    let error = verify_rate_leaf(&hash_tree, &rate)
        .expect_err("a different Candid rate must not satisfy the witness");

    assert!(matches!(
        error,
        CmcHostError::InvalidCertifiedRate { reason }
            if reason.contains("does not match")
    ));
}

#[test]
fn rejects_a_missing_rate_leaf() {
    let hash_tree = label(b"OTHER".to_vec(), leaf(vec![0]));
    let error = verify_rate_leaf(&hash_tree, &rate())
        .expect_err("the native CMC witness label is required");

    assert!(matches!(
        error,
        CmcHostError::InvalidCertifiedRate { reason }
            if reason.contains("leaf is absent")
    ));
}

#[test]
fn authenticates_the_certificate_before_accepting_the_rate() {
    let canister = cmc_canister();
    let rate = rate();
    let encoded_rate = ic_platform::candid::contract::encode_one(&rate).expect("encode CMC rate");
    let hash_tree = label(ICP_XDR_CONVERSION_RATE_LABEL.to_vec(), leaf(encoded_rate));
    let certificate = certificate_for_tree(&canister, hash_tree.digest().to_vec());
    let response = response(
        ic_platform::certification::cbor::encode_hash_tree(&hash_tree, 64 * 1024)
            .expect("encode hash tree"),
        ic_platform::certification::cbor::encode_certificate_untagged(&certificate, 64 * 1024)
            .expect("encode certificate"),
    );
    let agent = ic_platform::agent::Agent::builder()
        .with_url(super::super::DEFAULT_CMC_SOURCE_ENDPOINT)
        .build()
        .expect("build agent");

    let error = verified_certified_rate(&agent, &canister, response)
        .expect_err("an unsigned certificate must fail authentication");

    assert!(matches!(
        error,
        CmcHostError::CertificateAuthentication { .. }
    ));
}

fn rate() -> IcpXdrConversionRate {
    IcpXdrConversionRate {
        timestamp_seconds: 1_722_510_000,
        xdr_permyriad_per_icp: 49_164,
    }
}

fn response(hash_tree: Vec<u8>, certificate: Vec<u8>) -> IcpXdrConversionRateResponse {
    IcpXdrConversionRateResponse {
        data: ic_platform::cmc_types::IcpXdrConversionRate {
            timestamp_seconds: rate().timestamp_seconds,
            xdr_permyriad_per_icp: rate().xdr_permyriad_per_icp,
        },
        hash_tree,
        certificate,
    }
}

fn cmc_canister() -> Principal {
    Principal::from_text(MAINNET_CMC_CANISTER_ID).expect("valid CMC canister")
}

fn certificate_for_tree(canister: &Principal, certified_data: Vec<u8>) -> Certificate {
    Certificate {
        tree: label(
            b"canister".to_vec(),
            label(
                canister.as_slice().to_vec(),
                label(b"certified_data".to_vec(), leaf(certified_data)),
            ),
        ),
        signature: Vec::new(),
        delegation: None,
    }
}

#[test]
fn certified_rate_leaf_keeps_the_external_candid_bytes() {
    // Independent didc encoding of the official two-nat64 rate record.
    let expected = crate::hex::decode_hex(
        "4449444c016c02dff581a00878d6d5dac60f7801000cc0000000000000b06aab6600000000",
    )
    .unwrap();
    assert_eq!(
        ic_platform::candid::contract::encode_one(rate()).unwrap(),
        expected
    );
    let decoded: IcpXdrConversionRate = crate::candid_decode::decode_reply(&expected).unwrap();
    assert_eq!(decoded, rate());
}
