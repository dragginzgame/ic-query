use crate::sns::report::tests::*;

pub(in crate::sns::report::tests) const ROOT_A: &str = "be2us-64aaa-aaaaa-qaabq-cai";
pub(in crate::sns::report::tests) const GOVERNANCE_A: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";
pub(in crate::sns::report::tests) const LEDGER_A: &str = "bd3sg-teaaa-aaaaa-qaaba-cai";
const SWAP_A: &str = "br5f7-7uaaa-aaaaa-qaaca-cai";
pub(in crate::sns::report::tests) const INDEX_A: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const ROOT_B: &str = "bd3sg-teaaa-aaaaa-qaaba-cai";
const GOVERNANCE_B: &str = "br5f7-7uaaa-aaaaa-qaaca-cai";
const LEDGER_B: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const SWAP_B: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const INDEX_B: &str = "r7inp-6aaaa-aaaaa-aaabq-cai";

pub(in crate::sns::report::tests) struct FixtureSnsListSource;

impl SnsListSource for FixtureSnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        Ok(MainnetSnsList {
            network: MAINNET_NETWORK.to_string(),
            sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            sns_instances: vec![fixture_sns(
                "Fixture SNS",
                Some("Fixture description"),
                Some("https://example.com"),
                ROOT_A,
                GOVERNANCE_A,
                LEDGER_A,
                SWAP_A,
                INDEX_A,
                None,
            )],
        })
    }
}

pub(in crate::sns::report::tests) struct UnsortedFixtureSnsListSource;

impl SnsListSource for UnsortedFixtureSnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        Ok(MainnetSnsList {
            network: MAINNET_NETWORK.to_string(),
            sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            sns_instances: vec![
                fixture_sns(
                    "A Name",
                    None,
                    None,
                    ROOT_A,
                    GOVERNANCE_A,
                    LEDGER_A,
                    SWAP_A,
                    INDEX_A,
                    None,
                ),
                fixture_sns(
                    "Z Name",
                    None,
                    None,
                    ROOT_B,
                    GOVERNANCE_B,
                    LEDGER_B,
                    SWAP_B,
                    INDEX_B,
                    None,
                ),
            ],
        })
    }
}

pub(in crate::sns::report::tests) struct MetadataErrorFixtureSnsListSource;

impl SnsListSource for MetadataErrorFixtureSnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        Ok(MainnetSnsList {
            network: MAINNET_NETWORK.to_string(),
            sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            sns_instances: vec![fixture_sns(
                "unnamed-be2us",
                None,
                None,
                ROOT_A,
                GOVERNANCE_A,
                LEDGER_A,
                SWAP_A,
                INDEX_A,
                Some("get_metadata: Canister has no Wasm module"),
            )],
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn fixture_sns(
    name: &str,
    description: Option<&str>,
    url: Option<&str>,
    root_canister_id: &str,
    governance_canister_id: &str,
    ledger_canister_id: &str,
    swap_canister_id: &str,
    index_canister_id: &str,
    metadata_error: Option<&str>,
) -> MainnetSns {
    MainnetSns {
        id: 0,
        name: name.to_string(),
        description: description.map(str::to_string),
        url: url.map(str::to_string),
        root_canister_id: root_canister_id.to_string(),
        governance_canister_id: governance_canister_id.to_string(),
        ledger_canister_id: ledger_canister_id.to_string(),
        swap_canister_id: swap_canister_id.to_string(),
        index_canister_id: index_canister_id.to_string(),
        metadata_error: metadata_error.map(str::to_string),
    }
}
