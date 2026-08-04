//! Module: sns::report::source::model::list
//!
//! Responsibility: source-layer SNS-W inventory, catalog enrichment, and joined identity models.
//! Does not own: live transport, lookup parsing, report assembly, or rendering.
//! Boundary: validates discovery provenance and exact enrichment target coverage before joining.

use super::{SnsSourceRequest, validation::SnsSourceValidator};
use crate::sns::report::{MAINNET_SNS_WASM_CANISTER_ID, SnsHostError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const COMPACT_PRINCIPAL_CHARS: usize = 5;
const INVENTORY_VALIDATOR: SnsSourceValidator =
    SnsSourceValidator::new("SNS-W deployed SNS inventory");
const METADATA_VALIDATOR: SnsSourceValidator = SnsSourceValidator::new("SNS metadata");
const LIFECYCLE_VALIDATOR: SnsSourceValidator = SnsSourceValidator::new("SNS lifecycle");

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
/// MainnetSnsLifecycle
///
/// Raw Swap lifecycle result for one explicitly requested deployed SNS.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsLifecycle {
    /// Root canister identity that keys this result to the deployed-SNS inventory.
    pub root_canister_id: String,
    /// Native Swap lifecycle discriminant.
    pub lifecycle: Option<i32>,
    /// Stable native lifecycle label derived from `lifecycle`.
    pub lifecycle_name: Option<String>,
    /// Bounded lifecycle query failure retained instead of dropping the SNS row.
    pub lifecycle_error: Option<String>,
}

///
/// MainnetSns
///
/// Joined deployed SNS identity, metadata, and optional lifecycle evidence.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
    pub lifecycle: Option<i32>,
    pub lifecycle_name: Option<String>,
    pub lifecycle_error: Option<String>,
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

pub(in crate::sns::report) fn validate_joined_mainnet_sns_inventory(
    inventory: &JoinedMainnetSnsInventory,
) -> Result<(), SnsHostError> {
    let mut roots = BTreeSet::new();
    for (index, sns) in inventory.sns_instances.iter().enumerate() {
        let expected_id = index + 1;
        if sns.id != expected_id {
            return Err(INVENTORY_VALIDATOR
                .invalid(format!("SNS list id is {}, expected {expected_id}", sns.id)));
        }
        validate_sns_canisters(&MainnetSnsCanisters {
            root_canister_id: sns.root_canister_id.clone(),
            governance_canister_id: sns.governance_canister_id.clone(),
            ledger_canister_id: sns.ledger_canister_id.clone(),
            swap_canister_id: sns.swap_canister_id.clone(),
            index_canister_id: sns.index_canister_id.clone(),
        })?;
        if !roots.insert(sns.root_canister_id.as_str()) {
            return Err(INVENTORY_VALIDATOR.invalid(format!(
                "duplicate root canister id {}",
                sns.root_canister_id
            )));
        }
        validate_metadata_text(&sns.root_canister_id, "name", &sns.name)?;
        validate_optional_metadata_text(
            &sns.root_canister_id,
            "description",
            sns.description.as_deref(),
        )?;
        validate_optional_metadata_text(&sns.root_canister_id, "url", sns.url.as_deref())?;
        validate_optional_metadata_text(
            &sns.root_canister_id,
            "metadata_error",
            sns.metadata_error.as_deref(),
        )?;
        validate_joined_lifecycle(sns, false)?;
    }
    Ok(())
}

pub(in crate::sns::report) fn validate_joined_mainnet_sns_catalog(
    inventory: &JoinedMainnetSnsInventory,
) -> Result<(), SnsHostError> {
    validate_joined_mainnet_sns_inventory(inventory)?;
    for sns in &inventory.sns_instances {
        validate_joined_lifecycle(sns, true)?;
    }
    Ok(())
}

pub(in crate::sns::report) fn join_mainnet_sns_lifecycles(
    inventory: &mut JoinedMainnetSnsInventory,
    lifecycles: Vec<MainnetSnsLifecycle>,
) -> Result<(), SnsHostError> {
    validate_mainnet_sns_lifecycles(&inventory.sns_instances, &lifecycles)?;
    let mut lifecycle_by_root = lifecycles
        .into_iter()
        .map(|lifecycle| (lifecycle.root_canister_id.clone(), lifecycle))
        .collect::<BTreeMap<_, _>>();
    for sns in &mut inventory.sns_instances {
        let lifecycle = lifecycle_by_root
            .remove(&sns.root_canister_id)
            .expect("exact lifecycle validation requires every inventory root");
        sns.lifecycle = lifecycle.lifecycle;
        sns.lifecycle_name = lifecycle.lifecycle_name;
        sns.lifecycle_error = lifecycle.lifecycle_error;
    }
    Ok(())
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

fn validate_mainnet_sns_lifecycles(
    targets: &[MainnetSns],
    lifecycles: &[MainnetSnsLifecycle],
) -> Result<(), SnsHostError> {
    let expected_roots = targets
        .iter()
        .map(|sns| sns.root_canister_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut actual_roots = BTreeSet::new();
    for row in lifecycles {
        LIFECYCLE_VALIDATOR
            .canonical_principal("lifecycle root_canister_id", &row.root_canister_id)?;
        if !actual_roots.insert(row.root_canister_id.as_str()) {
            return Err(LIFECYCLE_VALIDATOR.invalid(format!(
                "duplicate lifecycle root canister id {}",
                row.root_canister_id
            )));
        }
        if !expected_roots.contains(row.root_canister_id.as_str()) {
            return Err(LIFECYCLE_VALIDATOR.invalid(format!(
                "lifecycle returned unrequested root canister id {}",
                row.root_canister_id
            )));
        }
        validate_lifecycle_fields(
            &row.root_canister_id,
            row.lifecycle,
            row.lifecycle_name.as_deref(),
            row.lifecycle_error.as_deref(),
            true,
        )?;
    }
    if actual_roots != expected_roots {
        let missing = expected_roots
            .difference(&actual_roots)
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(LIFECYCLE_VALIDATOR.invalid(format!(
            "lifecycle is missing requested root canister ids: {missing}"
        )));
    }
    Ok(())
}

fn validate_joined_lifecycle(sns: &MainnetSns, required: bool) -> Result<(), SnsHostError> {
    validate_lifecycle_fields(
        &sns.root_canister_id,
        sns.lifecycle,
        sns.lifecycle_name.as_deref(),
        sns.lifecycle_error.as_deref(),
        required,
    )
}

fn validate_lifecycle_fields(
    root_canister_id: &str,
    lifecycle: Option<i32>,
    lifecycle_name: Option<&str>,
    lifecycle_error: Option<&str>,
    required: bool,
) -> Result<(), SnsHostError> {
    if let Some(error) = lifecycle_error {
        validate_lifecycle_text(root_canister_id, "lifecycle_error", error)?;
        if lifecycle.is_some() || lifecycle_name.is_some() {
            return Err(LIFECYCLE_VALIDATOR.invalid(format!(
                "lifecycle for {root_canister_id} contains both value fields and lifecycle_error"
            )));
        }
        return Ok(());
    }
    if let Some(lifecycle) = lifecycle {
        let expected_name = super::swap::sns_swap_lifecycle_name(Some(lifecycle));
        if lifecycle_name != expected_name {
            return Err(LIFECYCLE_VALIDATOR.invalid(format!(
                "lifecycle_name is {lifecycle_name:?}, expected {expected_name:?} for lifecycle {lifecycle}"
            )));
        }
        return Ok(());
    }
    if lifecycle_name.is_some() {
        return Err(LIFECYCLE_VALIDATOR.invalid(format!(
            "lifecycle for {root_canister_id} has a name without a raw value"
        )));
    }
    if required {
        return Err(LIFECYCLE_VALIDATOR.invalid(format!(
            "lifecycle for {root_canister_id} has neither a value nor lifecycle_error"
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

fn validate_lifecycle_text(
    root_canister_id: &str,
    field: &'static str,
    value: &str,
) -> Result<(), SnsHostError> {
    if value.trim().is_empty() {
        return Err(LIFECYCLE_VALIDATOR
            .invalid(format!("lifecycle {field} for {root_canister_id} is empty")));
    }
    if value.trim() != value {
        return Err(LIFECYCLE_VALIDATOR.invalid(format!(
            "lifecycle {field} for {root_canister_id} has surrounding whitespace"
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
        lifecycle: None,
        lifecycle_name: None,
        lifecycle_error: None,
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
