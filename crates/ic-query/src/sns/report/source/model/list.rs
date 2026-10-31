//! Module: sns::report::source::model::list
//!
//! Responsibility: source-layer SNS-W inventory, catalog enrichment, and joined identity models.
//! Does not own: live transport, lookup parsing, report assembly, or rendering.
//! Boundary: validates discovery provenance and exact enrichment target coverage before joining.

use super::validation::SnsSourceValidator;
use crate::report::CollectionSourceRequest;
use crate::sns::report::{MAINNET_SNS_WASM_CANISTER_ID, MainnetSnsCanisters, SnsHostError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const COMPACT_PRINCIPAL_CHARS: usize = 5;
const INVENTORY_VALIDATOR: SnsSourceValidator =
    SnsSourceValidator::new("SNS-W deployed SNS inventory");
const METADATA_VALIDATOR: SnsSourceValidator = SnsSourceValidator::new("SNS metadata");
const LIFECYCLE_VALIDATOR: SnsSourceValidator = SnsSourceValidator::new("SNS lifecycle");

enum EnrichmentKind {
    Metadata,
    Lifecycle,
}

impl EnrichmentKind {
    const fn context(self) -> (&'static str, &'static str, SnsSourceValidator) {
        match self {
            Self::Metadata => ("metadata", "metadata root_canister_id", METADATA_VALIDATOR),
            Self::Lifecycle => (
                "lifecycle",
                "lifecycle root_canister_id",
                LIFECYCLE_VALIDATOR,
            ),
        }
    }
    fn validate_text(
        self,
        root_canister_id: &str,
        field: &'static str,
        value: &str,
    ) -> Result<(), SnsHostError> {
        let (kind, _, validator) = self.context();
        if value.trim().is_empty() {
            return Err(
                validator.invalid(format!("{kind} {field} for {root_canister_id} is empty"))
            );
        }
        if value.trim() != value {
            return Err(validator.invalid(format!(
                "{kind} {field} for {root_canister_id} has surrounding whitespace"
            )));
        }
        Ok(())
    }
}

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
/// MainnetSnsMetadata
///
/// Metadata result for one explicitly requested deployed SNS.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsMetadata {
    /// Root canister identity that keys this metadata result to inventory.
    pub root_canister_id: String,
    /// Optional metadata fields on success, or bounded query failure evidence.
    pub outcome: Result<MainnetSnsMetadataFields, String>,
}

///
/// MainnetSnsMetadataFields
///
/// Optional fields returned by a successful Governance metadata query.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsMetadataFields {
    /// Optional project name.
    pub name: Option<String>,
    /// Optional project description.
    pub description: Option<String>,
    /// Optional project URL.
    pub url: Option<String>,
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
    /// Raw lifecycle code on success, or bounded query failure evidence.
    pub outcome: Result<i32, String>,
}

///
/// MainnetSns
///
/// Joined deployed SNS identity, metadata, and optional lifecycle evidence.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MainnetSns {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    /// Complete SNS-W canister identity, preserved flat in serialized catalogs.
    #[serde(flatten)]
    pub canisters: MainnetSnsCanisters,
    pub metadata_error: Option<String>,
    /// Native Swap lifecycle discriminant, when the lifecycle query succeeded.
    pub lifecycle: Option<i32>,
    /// Bounded lifecycle query failure retained instead of dropping the SNS row.
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
    request: &CollectionSourceRequest,
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
        validate_sns_canister_ids(
            &sns.root_canister_id,
            &sns.governance_canister_id,
            &sns.ledger_canister_id,
            &sns.swap_canister_id,
            &sns.index_canister_id,
        )?;
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
    let mut metadata_by_root = index_mainnet_sns_metadata(&inventory.sns_instances, metadata)?;
    let sns_instances = inventory
        .sns_instances
        .into_iter()
        .map(|canisters| join_indexed_mainnet_sns(canisters, &mut metadata_by_root))
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

pub(in crate::sns::report) fn join_selected_mainnet_sns(
    target: MainnetSnsCanisters,
    metadata: Vec<MainnetSnsMetadata>,
) -> Result<MainnetSns, SnsHostError> {
    let mut metadata_by_root = index_mainnet_sns_metadata(std::slice::from_ref(&target), metadata)?;
    join_indexed_mainnet_sns(target, &mut metadata_by_root)
}

fn join_indexed_mainnet_sns(
    canisters: MainnetSnsCanisters,
    metadata_by_root: &mut BTreeMap<String, MainnetSnsMetadata>,
) -> Result<MainnetSns, SnsHostError> {
    let metadata = metadata_by_root
        .remove(&canisters.root_canister_id)
        .ok_or_else(|| {
            METADATA_VALIDATOR.invalid(format!(
                "metadata is missing requested root canister id {}",
                canisters.root_canister_id
            ))
        })?;
    Ok(joined_mainnet_sns(canisters, metadata))
}

pub(in crate::sns::report) fn validate_joined_mainnet_sns_catalog(
    sns_instances: &[MainnetSns],
) -> Result<(), SnsHostError> {
    let mut roots = BTreeSet::new();
    for (index, sns) in sns_instances.iter().enumerate() {
        let expected_id = index + 1;
        if sns.id != expected_id {
            return Err(INVENTORY_VALIDATOR
                .invalid(format!("SNS list id is {}, expected {expected_id}", sns.id)));
        }
        validate_sns_canister_ids(
            &sns.canisters.root_canister_id,
            &sns.canisters.governance_canister_id,
            &sns.canisters.ledger_canister_id,
            &sns.canisters.swap_canister_id,
            &sns.canisters.index_canister_id,
        )?;
        if !roots.insert(sns.canisters.root_canister_id.as_str()) {
            return Err(INVENTORY_VALIDATOR.invalid(format!(
                "duplicate root canister id {}",
                sns.canisters.root_canister_id
            )));
        }
        EnrichmentKind::Metadata.validate_text(
            &sns.canisters.root_canister_id,
            "name",
            &sns.name,
        )?;
        validate_optional_metadata_text(
            &sns.canisters.root_canister_id,
            "description",
            sns.description.as_deref(),
        )?;
        validate_optional_metadata_text(
            &sns.canisters.root_canister_id,
            "url",
            sns.url.as_deref(),
        )?;
        validate_optional_metadata_text(
            &sns.canisters.root_canister_id,
            "metadata_error",
            sns.metadata_error.as_deref(),
        )?;
        validate_lifecycle_fields(
            &sns.canisters.root_canister_id,
            sns.lifecycle,
            sns.lifecycle_error.as_deref(),
        )?;
    }
    Ok(())
}

pub(in crate::sns::report) fn join_mainnet_sns_lifecycles(
    inventory: &mut JoinedMainnetSnsInventory,
    lifecycles: Vec<MainnetSnsLifecycle>,
) -> Result<(), SnsHostError> {
    let mut lifecycle_by_root = index_mainnet_sns_lifecycles(&inventory.sns_instances, lifecycles)?;
    for sns in &mut inventory.sns_instances {
        let lifecycle = lifecycle_by_root
            .remove(&sns.canisters.root_canister_id)
            .expect("exact lifecycle validation requires every inventory root");
        (sns.lifecycle, sns.lifecycle_error) = match lifecycle.outcome {
            Ok(value) => (Some(value), None),
            Err(error) => (None, Some(error)),
        };
    }
    Ok(())
}

fn index_mainnet_sns_metadata(
    targets: &[MainnetSnsCanisters],
    metadata: Vec<MainnetSnsMetadata>,
) -> Result<BTreeMap<String, MainnetSnsMetadata>, SnsHostError> {
    index_enrichment_rows(
        targets
            .iter()
            .map(|sns| sns.root_canister_id.as_str())
            .collect(),
        metadata,
        EnrichmentKind::Metadata,
        |row| &row.root_canister_id,
        |row| {
            match &row.outcome {
                Ok(fields) => {
                    for (field, value) in [
                        ("name", fields.name.as_deref()),
                        ("description", fields.description.as_deref()),
                        ("url", fields.url.as_deref()),
                    ] {
                        validate_optional_metadata_text(&row.root_canister_id, field, value)?;
                    }
                }
                Err(error) => EnrichmentKind::Metadata.validate_text(
                    &row.root_canister_id,
                    "metadata_error",
                    error,
                )?,
            }
            Ok(())
        },
    )
}

fn index_mainnet_sns_lifecycles(
    targets: &[MainnetSns],
    lifecycles: Vec<MainnetSnsLifecycle>,
) -> Result<BTreeMap<String, MainnetSnsLifecycle>, SnsHostError> {
    index_enrichment_rows(
        targets
            .iter()
            .map(|sns| sns.canisters.root_canister_id.as_str())
            .collect(),
        lifecycles,
        EnrichmentKind::Lifecycle,
        |row| &row.root_canister_id,
        |row| {
            if let Err(error) = &row.outcome {
                EnrichmentKind::Lifecycle.validate_text(
                    &row.root_canister_id,
                    "lifecycle_error",
                    error,
                )?;
            }
            Ok(())
        },
    )
}

fn index_enrichment_rows<Row>(
    mut expected_roots: BTreeSet<&str>,
    rows: Vec<Row>,
    kind: EnrichmentKind,
    root: impl Fn(&Row) -> &str,
    validate: impl Fn(&Row) -> Result<(), SnsHostError>,
) -> Result<BTreeMap<String, Row>, SnsHostError> {
    let (kind, field, validator) = kind.context();
    let mut by_root = BTreeMap::new();
    for row in rows {
        let root = root(&row);
        validator.canonical_principal(field, root)?;
        if by_root.contains_key(root) {
            return Err(validator.invalid(format!("duplicate {kind} root canister id {root}")));
        }
        if !expected_roots.remove(root) {
            return Err(validator.invalid(format!(
                "{kind} returned unrequested root canister id {root}"
            )));
        }
        validate(&row)?;
        by_root.insert(root.to_string(), row);
    }
    if !expected_roots.is_empty() {
        let missing = expected_roots.into_iter().collect::<Vec<_>>().join(", ");
        return Err(validator.invalid(format!(
            "{kind} is missing requested root canister ids: {missing}"
        )));
    }
    Ok(by_root)
}

fn validate_lifecycle_fields(
    root_canister_id: &str,
    lifecycle: Option<i32>,

    lifecycle_error: Option<&str>,
) -> Result<(), SnsHostError> {
    if let Some(error) = lifecycle_error {
        EnrichmentKind::Lifecycle.validate_text(root_canister_id, "lifecycle_error", error)?;
        if lifecycle.is_some() {
            return Err(LIFECYCLE_VALIDATOR.invalid(format!(
                "lifecycle for {root_canister_id} contains both value fields and lifecycle_error"
            )));
        }
        return Ok(());
    }
    if lifecycle.is_some() {
        return Ok(());
    }
    Err(LIFECYCLE_VALIDATOR.invalid(format!(
        "lifecycle for {root_canister_id} has neither a value nor lifecycle_error"
    )))
}

fn validate_optional_metadata_text(
    root_canister_id: &str,
    field: &'static str,
    value: Option<&str>,
) -> Result<(), SnsHostError> {
    if let Some(value) = value {
        EnrichmentKind::Metadata.validate_text(root_canister_id, field, value)?;
    }
    Ok(())
}

fn joined_mainnet_sns(canisters: MainnetSnsCanisters, metadata: MainnetSnsMetadata) -> MainnetSns {
    let (fields, metadata_error) = match metadata.outcome {
        Ok(fields) => (fields, None),
        Err(error) => (
            MainnetSnsMetadataFields {
                name: None,
                description: None,
                url: None,
            },
            Some(error),
        ),
    };
    let name = fields.name.unwrap_or_else(|| {
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
        description: fields.description,
        url: fields.url,
        canisters,
        metadata_error,
        lifecycle: None,

        lifecycle_error: None,
    }
}

fn validate_sns_canister_ids(
    root_canister_id: &str,
    governance_canister_id: &str,
    ledger_canister_id: &str,
    swap_canister_id: &str,
    index_canister_id: &str,
) -> Result<(), SnsHostError> {
    for (field, value) in [
        ("root_canister_id", root_canister_id),
        ("governance_canister_id", governance_canister_id),
        ("ledger_canister_id", ledger_canister_id),
        ("swap_canister_id", swap_canister_id),
        ("index_canister_id", index_canister_id),
    ] {
        INVENTORY_VALIDATOR.canonical_principal(field, value)?;
    }
    Ok(())
}
