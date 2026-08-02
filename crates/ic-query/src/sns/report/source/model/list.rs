//! Module: sns::report::source::model::list
//!
//! Responsibility: source-layer SNS-W inventory, targeted metadata, and joined identity models.
//! Does not own: live transport, lookup parsing, report assembly, or rendering.
//! Boundary: validates discovery provenance and exact metadata target coverage before joining.

use super::{SnsSourceRequest, validation::SnsSourceValidator};
use crate::sns::report::{MAINNET_SNS_WASM_CANISTER_ID, SnsHostError};
use std::collections::{BTreeMap, BTreeSet};

const COMPACT_PRINCIPAL_CHARS: usize = 5;
const INVENTORY_VALIDATOR: SnsSourceValidator =
    SnsSourceValidator::new("SNS-W deployed SNS inventory");
const METADATA_VALIDATOR: SnsSourceValidator = SnsSourceValidator::new("SNS metadata");

///
/// MainnetSnsInventory
///
/// Unenriched deployed-SNS canister inventory returned by mainnet SNS-W.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsInventory {
    /// Requested network identity echoed by the source.
    pub network: String,
    /// SNS-W canister queried for the inventory.
    pub sns_wasm_canister_id: String,
    /// Collection timestamp supplied by the source request.
    pub fetched_at: String,
    /// Collector identity supplied by the source request.
    pub fetched_by: String,
    /// IC API endpoint used for the inventory query.
    pub source_endpoint: String,
    /// Deployed SNS canister sets in authoritative SNS-W response order.
    pub sns_instances: Vec<MainnetSnsCanisters>,
}

///
/// MainnetSnsCanisters
///
/// Native canister identities for one deployed SNS-W inventory row.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsCanisters {
    /// SNS Root canister principal.
    pub root_canister_id: String,
    /// SNS Governance canister principal.
    pub governance_canister_id: String,
    /// SNS ledger canister principal.
    pub ledger_canister_id: String,
    /// SNS decentralization-swap canister principal.
    pub swap_canister_id: String,
    /// SNS ledger index canister principal.
    pub index_canister_id: String,
}

///
/// MainnetSnsMetadata
///
/// Metadata result for one explicitly requested deployed SNS.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsMetadata {
    /// Root canister identity that keys this metadata result to inventory.
    pub root_canister_id: String,
    /// Optional SNS name returned by Governance.
    pub name: Option<String>,
    /// Optional SNS description returned by Governance.
    pub description: Option<String>,
    /// Optional SNS project URL returned by Governance.
    pub url: Option<String>,
    /// Compact metadata query failure retained instead of dropping the SNS row.
    pub metadata_error: Option<String>,
}

///
/// MainnetSns
///
/// Joined deployed SNS identity and optional metadata used by report capabilities.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSns {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
}

///
/// JoinedMainnetSnsInventory
///
/// Internal joined inventory used by list and direct-report assembly.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct JoinedMainnetSnsInventory {
    pub(in crate::sns::report) network: String,
    pub(in crate::sns::report) sns_wasm_canister_id: String,
    pub(in crate::sns::report) fetched_at: String,
    pub(in crate::sns::report) fetched_by: String,
    pub(in crate::sns::report) source_endpoint: String,
    pub(in crate::sns::report) sns_instances: Vec<MainnetSns>,
}

pub(in crate::sns::report) fn validate_mainnet_sns_inventory(
    request: &SnsSourceRequest,
    inventory: &MainnetSnsInventory,
) -> Result<(), SnsHostError> {
    INVENTORY_VALIDATOR.exact("network", &request.network, &inventory.network)?;
    INVENTORY_VALIDATOR.exact(
        "sns_wasm_canister_id",
        MAINNET_SNS_WASM_CANISTER_ID,
        &inventory.sns_wasm_canister_id,
    )?;
    INVENTORY_VALIDATOR.exact("fetched_at", &request.fetched_at, &inventory.fetched_at)?;
    INVENTORY_VALIDATOR.exact("fetched_by", &request.fetched_by, &inventory.fetched_by)?;
    INVENTORY_VALIDATOR.exact(
        "source_endpoint",
        &request.endpoint,
        &inventory.source_endpoint,
    )?;

    let mut roots = BTreeSet::new();
    for sns in &inventory.sns_instances {
        validate_sns_canisters(sns)?;
        if !roots.insert(sns.root_canister_id.as_str()) {
            return Err(INVENTORY_VALIDATOR.invalid(format!(
                "duplicate root canister id {}",
                sns.root_canister_id
            )));
        }
    }
    Ok(())
}

pub(in crate::sns::report) fn join_mainnet_sns_inventory(
    inventory: MainnetSnsInventory,
    metadata: Vec<MainnetSnsMetadata>,
) -> Result<JoinedMainnetSnsInventory, SnsHostError> {
    validate_mainnet_sns_metadata(&inventory.sns_instances, &metadata)?;
    let mut metadata_by_root = metadata
        .into_iter()
        .map(|metadata| (metadata.root_canister_id.clone(), metadata))
        .collect::<BTreeMap<_, _>>();
    let sns_instances = inventory
        .sns_instances
        .into_iter()
        .map(|canisters| {
            let root_canister_id = canisters.root_canister_id.clone();
            let metadata = metadata_by_root.remove(&root_canister_id).ok_or_else(|| {
                METADATA_VALIDATOR.invalid(format!(
                    "metadata is missing requested root canister id {root_canister_id}"
                ))
            })?;
            Ok(joined_mainnet_sns(canisters, metadata))
        })
        .collect::<Result<Vec<_>, SnsHostError>>()?;
    Ok(JoinedMainnetSnsInventory {
        network: inventory.network,
        sns_wasm_canister_id: inventory.sns_wasm_canister_id,
        fetched_at: inventory.fetched_at,
        fetched_by: inventory.fetched_by,
        source_endpoint: inventory.source_endpoint,
        sns_instances,
    })
}

fn validate_mainnet_sns_metadata(
    targets: &[MainnetSnsCanisters],
    metadata: &[MainnetSnsMetadata],
) -> Result<(), SnsHostError> {
    let expected_roots = targets
        .iter()
        .map(|sns| sns.root_canister_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut actual_roots = BTreeSet::new();
    for row in metadata {
        METADATA_VALIDATOR
            .canonical_principal("metadata root_canister_id", &row.root_canister_id)?;
        if !actual_roots.insert(row.root_canister_id.as_str()) {
            return Err(METADATA_VALIDATOR.invalid(format!(
                "duplicate metadata root canister id {}",
                row.root_canister_id
            )));
        }
        if !expected_roots.contains(row.root_canister_id.as_str()) {
            return Err(METADATA_VALIDATOR.invalid(format!(
                "metadata returned unrequested root canister id {}",
                row.root_canister_id
            )));
        }
        for (field, value) in [
            ("name", row.name.as_deref()),
            ("description", row.description.as_deref()),
            ("url", row.url.as_deref()),
        ] {
            validate_optional_metadata_text(&row.root_canister_id, field, value)?;
        }
        if let Some(error) = row.metadata_error.as_deref() {
            validate_metadata_text(&row.root_canister_id, "metadata_error", error)?;
            if row.name.is_some() || row.description.is_some() || row.url.is_some() {
                return Err(METADATA_VALIDATOR.invalid(format!(
                    "metadata for {} contains both payload fields and metadata_error",
                    row.root_canister_id
                )));
            }
        }
    }
    if actual_roots != expected_roots {
        let missing = expected_roots
            .difference(&actual_roots)
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(METADATA_VALIDATOR.invalid(format!(
            "metadata is missing requested root canister ids: {missing}"
        )));
    }
    Ok(())
}

fn validate_optional_metadata_text(
    root_canister_id: &str,
    field: &'static str,
    value: Option<&str>,
) -> Result<(), SnsHostError> {
    if let Some(value) = value {
        validate_metadata_text(root_canister_id, field, value)?;
    }
    Ok(())
}

fn validate_metadata_text(
    root_canister_id: &str,
    field: &'static str,
    value: &str,
) -> Result<(), SnsHostError> {
    if value.trim().is_empty() {
        return Err(
            METADATA_VALIDATOR.invalid(format!("metadata {field} for {root_canister_id} is empty"))
        );
    }
    if value.trim() != value {
        return Err(METADATA_VALIDATOR.invalid(format!(
            "metadata {field} for {root_canister_id} has surrounding whitespace"
        )));
    }
    Ok(())
}

fn joined_mainnet_sns(canisters: MainnetSnsCanisters, metadata: MainnetSnsMetadata) -> MainnetSns {
    let name = metadata.name.unwrap_or_else(|| {
        format!(
            "unnamed-{}",
            canisters
                .root_canister_id
                .chars()
                .take(COMPACT_PRINCIPAL_CHARS)
                .collect::<String>()
        )
    });
    MainnetSns {
        id: 0,
        name,
        description: metadata.description,
        url: metadata.url,
        root_canister_id: canisters.root_canister_id,
        governance_canister_id: canisters.governance_canister_id,
        ledger_canister_id: canisters.ledger_canister_id,
        swap_canister_id: canisters.swap_canister_id,
        index_canister_id: canisters.index_canister_id,
        metadata_error: metadata.metadata_error,
    }
}

fn validate_sns_canisters(sns: &MainnetSnsCanisters) -> Result<(), SnsHostError> {
    for (field, value) in [
        ("root_canister_id", sns.root_canister_id.as_str()),
        (
            "governance_canister_id",
            sns.governance_canister_id.as_str(),
        ),
        ("ledger_canister_id", sns.ledger_canister_id.as_str()),
        ("swap_canister_id", sns.swap_canister_id.as_str()),
        ("index_canister_id", sns.index_canister_id.as_str()),
    ] {
        INVENTORY_VALIDATOR.canonical_principal(field, value)?;
    }
    Ok(())
}
