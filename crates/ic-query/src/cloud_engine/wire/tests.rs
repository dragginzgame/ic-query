use super::*;

#[test]
fn operator_resolution_shape_round_trips() {
    let args = GetEngineOperatorBySubnetArgs {
        subnet_id: Some(Principal::from_text("aaaaa-aa").unwrap()),
    };
    let encoded = ic_platform::candid::contract::encode_one(&args).expect("encode operator args");
    let decoded: GetEngineOperatorBySubnetArgs =
        crate::candid_decode::decode_reply(&encoded).expect("decode operator args");
    assert_eq!(decoded, args);

    let result = GetEngineOperatorBySubnetResult {
        engine_operator_id: Some(Principal::from_text("wlnge-zyaaa-aaabw-aaaaa-cai").unwrap()),
    };
    let encoded =
        ic_platform::candid::contract::encode_one(&result).expect("encode operator result");
    let decoded: GetEngineOperatorBySubnetResult =
        crate::candid_decode::decode_reply(&encoded).expect("decode operator result");

    assert_eq!(decoded, result);
}

#[test]
fn marketplace_node_classes_decode_exact_candid_labels() {
    // Independent Candid fixtures: one null variant with its wire label hash.
    for (expected, label, bytes) in [
        (
            CloudEngineNodeType::Type4_1,
            "type4_1",
            &[
                0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xac, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f, 0x01,
                0x00, 0x00,
            ][..],
        ),
        (
            CloudEngineNodeType::Type4_2,
            "type4_2",
            &[
                0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xad, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f, 0x01,
                0x00, 0x00,
            ][..],
        ),
        (
            CloudEngineNodeType::Type4_3,
            "type4_3",
            &[
                0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xae, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f, 0x01,
                0x00, 0x00,
            ][..],
        ),
        (
            CloudEngineNodeType::Type4_4,
            "type4_4",
            &[
                0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xaf, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f, 0x01,
                0x00, 0x00,
            ][..],
        ),
        (
            CloudEngineNodeType::Type4_5,
            "type4_5",
            &[
                0x44, 0x49, 0x44, 0x4c, 0x01, 0x6b, 0x01, 0xb0, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f, 0x01,
                0x00, 0x00,
            ][..],
        ),
    ] {
        let decoded: CloudEngineNodeType =
            crate::candid_decode::decode_reply(bytes).expect("wire node class");
        assert_eq!(decoded, expected);
        assert_eq!(serde_json::to_value(decoded).unwrap(), label);
        let encoded =
            ic_platform::candid::contract::encode_one(decoded).expect("encode node class");
        assert_eq!(
            crate::candid_decode::decode_reply::<CloudEngineNodeType>(&encoded).unwrap(),
            expected
        );
    }
}

#[test]
fn marketplace_shape_preserves_arbitrary_precision_cycle_amounts() {
    let row = CloudEngineMarketplaceEntryWire {
        key: "type4.1".to_string(),
        entry: CloudEnginePriceEntryWire {
            node_type: CloudEngineNodeType::Type4_1,
            dc: None,
            provider: None,
            price: CloudEnginePriceWire {
                net_cycles: Nat::from(471_065_106_452_u64),
                gross_cycles: Nat::from(588_831_383_065_u64),
            },
            updated_at: 1_785_946_128_242_156_275,
        },
    };
    let encoded =
        ic_platform::candid::contract::encode_one(vec![row]).expect("encode marketplace rows");
    let decoded: Vec<CloudEngineMarketplaceEntryWire> =
        crate::candid_decode::decode_reply(&encoded).expect("decode marketplace rows");
    let projected = decoded.into_iter().next().unwrap().into_report_row();

    assert_eq!(projected.node_type, CloudEngineNodeType::Type4_1);
    assert_eq!(projected.net_cycles_per_month, "471065106452");
}
