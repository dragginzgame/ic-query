use super::*;
use ic_platform::candid::contract::{CandidType, Principal};
use serde_json::json;

#[test]
fn canonical_accounts_validate_wire_lengths_and_preserve_explicit_subaccounts() {
    #[derive(CandidType)]
    #[candid_path("ic_platform::candid::contract")]
    struct UncheckedAccount {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    for length in [0, 31, 32, 33] {
        let bytes = ic_platform::candid::contract::encode_one(UncheckedAccount {
            owner: Principal::anonymous(),
            subaccount: Some(vec![0; length]),
        })
        .unwrap();
        assert_eq!(
            crate::candid_decode::decode_reply::<LedgerAccount>(&bytes).is_ok(),
            length == 32
        );
    }
    for (subaccount, expected) in [
        (None, json!(null)),
        (Some([0; 32]), json!("00".repeat(32))),
        (Some([0xff; 32]), json!("ff".repeat(32))),
    ] {
        let account = LedgerAccount {
            owner: Principal::anonymous(),
            subaccount,
        };
        assert_eq!(
            serde_json::to_value(Account(&account)).unwrap(),
            json!({
                "owner": "2vxsx-fae", "subaccount_hex": expected,
            })
        );
    }
}
