use super::super::*;
use super::{FixtureSnsListSource, GOVERNANCE_A, INDEX_A, LEDGER_A};

pub(in crate::sns::report::tests) struct FixtureSnsTokenSource;

impl SnsListSource for FixtureSnsTokenSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsTokenSource for FixtureSnsTokenSource {
    fn fetch_sns_token(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError> {
        assert_eq!(sns.ledger_canister_id, LEDGER_A);
        Ok(MainnetSnsToken {
            token_name: "Fixture Token".to_string(),
            token_symbol: "FIX".to_string(),
            decimals: 8,
            transfer_fee: "10_000".to_string(),
            total_supply: "1_000_000_000".to_string(),
            minting_account_owner: Some(GOVERNANCE_A.to_string()),
            minting_account_subaccount_hex: Some("000102".to_string()),
            ledger_index_canister_id: Some(INDEX_A.to_string()),
            ledger_index_error: None,
            supported_standards: vec![
                SnsTokenStandardRow {
                    name: "ICRC-1".to_string(),
                    url: "https://github.com/dfinity/ICRC-1".to_string(),
                },
                SnsTokenStandardRow {
                    name: "ICRC-2".to_string(),
                    url: "https://github.com/dfinity/ICRC-2".to_string(),
                },
            ],
            metadata: vec![
                SnsTokenMetadataRow {
                    key: "icrc1:name".to_string(),
                    value_type: "text".to_string(),
                    value: serde_json::json!("Fixture Token"),
                },
                SnsTokenMetadataRow {
                    key: "icrc1:decimals".to_string(),
                    value_type: "nat".to_string(),
                    value: serde_json::json!("8"),
                },
                SnsTokenMetadataRow {
                    key: "icrc1:fee".to_string(),
                    value_type: "nat".to_string(),
                    value: serde_json::json!("10_000"),
                },
                SnsTokenMetadataRow {
                    key: "icrc1:logo".to_string(),
                    value_type: "bool".to_string(),
                    value: serde_json::json!(true),
                },
            ],
        })
    }
}
