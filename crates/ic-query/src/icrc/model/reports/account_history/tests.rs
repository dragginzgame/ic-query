use super::IcrcAccountRow;
use serde_json::json;

#[test]
fn account_forms_round_trip_and_refuse_ambiguous_or_former_shapes() {
    for account in [
        IcrcAccountRow::Structured {
            owner: "aaaaa-aa".to_string(),
            subaccount_hex: None,
        },
        IcrcAccountRow::Legacy {
            account_identifier: "original-index-identifier".to_string(),
        },
    ] {
        let value = serde_json::to_value(&account).unwrap();
        assert_eq!(
            serde_json::from_value::<IcrcAccountRow>(value).unwrap(),
            account
        );
    }
    for value in [
        json!({"owner":"aaaaa-aa", "subaccount_hex":null, "account_identifier":null}),
        json!({"account_kind":"legacy", "account_identifier":"id", "owner":"aaaaa-aa"}),
        json!({"account_kind":"structured", "subaccount_hex":null}),
        json!({"account_kind":"structured", "owner":"aaaaa-aa", "account_identifier":"id"}),
    ] {
        assert!(serde_json::from_value::<IcrcAccountRow>(value).is_err());
    }
}
