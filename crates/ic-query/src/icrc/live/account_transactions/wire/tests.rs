use super::*;

#[test]
fn index_error_reply_preserves_target_and_message_in_both_codecs() {
    // External get_account_transactions reply: (variant { Err = record { message = "index unavailable" } }).
    let bytes = b"\x44\x49\x44\x4c\x02\x6c\x01\xc7\xeb\xc4\xd0\x09\x71\x6b\x01\xc5\xfe\xd2\x01\x00\x01\x01\x00\x11\x69\x6e\x64\x65\x78\x20\x75\x6e\x61\x76\x61\x69\x6c\x61\x62\x6c\x65";
    let index = Principal::anonymous();
    for result in [generic::decode(bytes, &index), icp::decode(bytes, &index)] {
        let result = result.expect("decode external index error reply");
        assert!(matches!(
            result,
            Err(IcrcAccountTransactionError::IndexQuery { index_canister_id, message })
                if index_canister_id == index.to_text() && message == "index unavailable"
        ));
    }
}
