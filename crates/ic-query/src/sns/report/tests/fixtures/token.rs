use super::{FixtureSnsDiscoverySource, GOVERNANCE_A, INDEX_A, LEDGER_A};
use crate::icrc::{IcrcTokenData, IcrcTokenMetadataRow, IcrcTokenStandardRow};
use crate::report::CollectionSourceRequest;
use crate::sns::report::tests::*;

///
/// FixtureSnsTokenSource
///
/// Successful SNS ledger token source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsTokenSource;

delegate_sns_discovery!(FixtureSnsTokenSource);

impl SnsTokenSource for FixtureSnsTokenSource {
    fn fetch_sns_token<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsToken, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(sns.canisters.ledger_canister_id, LEDGER_A);
            Ok(MainnetSnsToken {
                token: IcrcTokenData {
                    token_name: "Fixture Token".to_string(),
                    token_symbol: "FIX".to_string(),
                    decimals: 8,
                    transfer_fee: "10_000".to_string(),
                    total_supply: "1_000_000_000".to_string(),
                    minting_account_owner: Some(GOVERNANCE_A.to_string()),
                    minting_account_subaccount_hex: Some("000102".to_string()),
                    supported_standards: vec![
                        IcrcTokenStandardRow {
                            name: "ICRC-1".to_string(),
                            url: "https://github.com/dfinity/ICRC-1".to_string(),
                        },
                        IcrcTokenStandardRow {
                            name: "ICRC-2".to_string(),
                            url: "https://github.com/dfinity/ICRC-2".to_string(),
                        },
                    ],
                    metadata: vec![
                        IcrcTokenMetadataRow {
                            key: "icrc1:name".to_string(),
                            value_type: IcrcMetadataValueKind::Text,
                            value: serde_json::json!("Fixture Token"),
                        },
                        IcrcTokenMetadataRow {
                            key: "icrc1:decimals".to_string(),
                            value_type: IcrcMetadataValueKind::Nat,
                            value: serde_json::json!("8"),
                        },
                        IcrcTokenMetadataRow {
                            key: "icrc1:fee".to_string(),
                            value_type: IcrcMetadataValueKind::Nat,
                            value: serde_json::json!("10_000"),
                        },
                        IcrcTokenMetadataRow {
                            key: "icrc1:logo".to_string(),
                            value_type: IcrcMetadataValueKind::Text,
                            value: serde_json::json!("data:image/png;base64,fixture"),
                        },
                    ],
                },
                ledger_index_canister_id: Some(INDEX_A.to_string()),
                ledger_index_error: None,
            })
        })
    }
}
