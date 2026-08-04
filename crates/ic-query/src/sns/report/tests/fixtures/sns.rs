use crate::sns::report::source::sns_swap_lifecycle_name;
use crate::sns::report::tests::*;

pub(in crate::sns::report::tests) const ROOT_A: &str = "be2us-64aaa-aaaaa-qaabq-cai";
pub(in crate::sns::report::tests) const GOVERNANCE_A: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";
pub(in crate::sns::report::tests) const LEDGER_A: &str = "bd3sg-teaaa-aaaaa-qaaba-cai";
pub(in crate::sns::report::tests) const SWAP_A: &str = "br5f7-7uaaa-aaaaa-qaaca-cai";
pub(in crate::sns::report::tests) const INDEX_A: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
pub(in crate::sns::report::tests) const ROOT_B: &str = "bd3sg-teaaa-aaaaa-qaaba-cai";
const GOVERNANCE_B: &str = "br5f7-7uaaa-aaaaa-qaaca-cai";
const LEDGER_B: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const SWAP_B: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const INDEX_B: &str = "r7inp-6aaaa-aaaaa-aaabq-cai";

///
/// FixtureSnsDiscoverySource
///
/// Successful SNS discovery source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsDiscoverySource;

impl SnsDiscoverySource for FixtureSnsDiscoverySource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        Ok(fixture_inventory(request, vec![fixture_canisters_a()]))
    }

    fn fetch_sns_metadata(
        &self,
        _request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        Ok(targets
            .iter()
            .map(|target| fixture_metadata(target, None))
            .collect())
    }
}

impl SnsCatalogSource for FixtureSnsDiscoverySource {
    fn fetch_sns_lifecycles(
        &self,
        _request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
        Ok(fixture_lifecycles(targets, |_| 3))
    }
}

///
/// UnsortedFixtureSnsDiscoverySource
///
/// SNS discovery source with deliberately unsorted metadata names for view tests.
///

pub(in crate::sns::report::tests) struct UnsortedFixtureSnsDiscoverySource;

impl SnsDiscoverySource for UnsortedFixtureSnsDiscoverySource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        Ok(fixture_inventory(
            request,
            vec![fixture_canisters_a(), fixture_canisters_b()],
        ))
    }

    fn fetch_sns_metadata(
        &self,
        _request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        Ok(targets
            .iter()
            .map(|target| {
                let mut metadata = fixture_metadata(target, None);
                if target.root_canister_id == ROOT_A {
                    metadata.name = Some("A Name".to_string());
                }
                metadata
            })
            .collect())
    }
}

impl SnsCatalogSource for UnsortedFixtureSnsDiscoverySource {
    fn fetch_sns_lifecycles(
        &self,
        _request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
        Ok(fixture_lifecycles(targets, |target| {
            if target.root_canister_id == ROOT_A {
                3
            } else {
                4
            }
        }))
    }
}

///
/// MetadataErrorFixtureSnsDiscoverySource
///
/// SNS discovery source carrying a metadata failure for fallback tests.
///

pub(in crate::sns::report::tests) struct MetadataErrorFixtureSnsDiscoverySource;

impl SnsDiscoverySource for MetadataErrorFixtureSnsDiscoverySource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        Ok(fixture_inventory(request, vec![fixture_canisters_a()]))
    }

    fn fetch_sns_metadata(
        &self,
        _request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        Ok(targets
            .iter()
            .map(|target| {
                fixture_metadata(target, Some("get_metadata: Canister has no Wasm module"))
            })
            .collect())
    }
}

impl SnsCatalogSource for MetadataErrorFixtureSnsDiscoverySource {
    fn fetch_sns_lifecycles(
        &self,
        _request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
        Ok(fixture_lifecycles(targets, |_| 3))
    }
}

fn fixture_inventory(
    request: &SnsSourceRequest,
    sns_instances: Vec<MainnetSnsCanisters>,
) -> MainnetSnsInventory {
    MainnetSnsInventory {
        network: MAINNET_NETWORK.to_string(),
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        sns_instances,
    }
}

pub(in crate::sns::report::tests) fn fixture_canisters_a() -> MainnetSnsCanisters {
    fixture_canisters(ROOT_A, GOVERNANCE_A, LEDGER_A, SWAP_A, INDEX_A)
}

pub(in crate::sns::report::tests) fn fixture_sns_a() -> MainnetSns {
    MainnetSns {
        id: 1,
        name: "Fixture SNS".to_string(),
        description: Some("Fixture description".to_string()),
        url: Some("https://example.com".to_string()),
        root_canister_id: ROOT_A.to_string(),
        governance_canister_id: GOVERNANCE_A.to_string(),
        ledger_canister_id: LEDGER_A.to_string(),
        swap_canister_id: SWAP_A.to_string(),
        index_canister_id: INDEX_A.to_string(),
        metadata_error: None,
        lifecycle: None,
        lifecycle_name: None,
        lifecycle_error: None,
    }
}

fn fixture_canisters_b() -> MainnetSnsCanisters {
    fixture_canisters(ROOT_B, GOVERNANCE_B, LEDGER_B, SWAP_B, INDEX_B)
}

fn fixture_canisters(
    root_canister_id: &str,
    governance_canister_id: &str,
    ledger_canister_id: &str,
    swap_canister_id: &str,
    index_canister_id: &str,
) -> MainnetSnsCanisters {
    MainnetSnsCanisters {
        root_canister_id: root_canister_id.to_string(),
        governance_canister_id: governance_canister_id.to_string(),
        ledger_canister_id: ledger_canister_id.to_string(),
        swap_canister_id: swap_canister_id.to_string(),
        index_canister_id: index_canister_id.to_string(),
    }
}

fn fixture_metadata(
    target: &MainnetSnsCanisters,
    metadata_error: Option<&str>,
) -> MainnetSnsMetadata {
    if let Some(metadata_error) = metadata_error {
        return MainnetSnsMetadata {
            root_canister_id: target.root_canister_id.clone(),
            name: None,
            description: None,
            url: None,
            metadata_error: Some(metadata_error.to_string()),
        };
    }
    let (name, description, url) = if target.root_canister_id == ROOT_A {
        (
            Some("Fixture SNS"),
            Some("Fixture description"),
            Some("https://example.com"),
        )
    } else {
        (Some("Z Name"), None, None)
    };
    MainnetSnsMetadata {
        root_canister_id: target.root_canister_id.clone(),
        name: Some(name.unwrap().to_string()),
        description: description.map(str::to_string),
        url: url.map(str::to_string),
        metadata_error: None,
    }
}

fn fixture_lifecycles(
    targets: &[MainnetSnsCanisters],
    lifecycle: impl Fn(&MainnetSnsCanisters) -> i32,
) -> Vec<MainnetSnsLifecycle> {
    targets
        .iter()
        .map(|target| {
            let lifecycle = lifecycle(target);
            MainnetSnsLifecycle {
                root_canister_id: target.root_canister_id.clone(),
                lifecycle: Some(lifecycle),
                lifecycle_name: sns_swap_lifecycle_name(Some(lifecycle)).map(str::to_string),
                lifecycle_error: None,
            }
        })
        .collect()
}
