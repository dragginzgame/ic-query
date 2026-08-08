//! Module: ic::api_boundary_node::host
//!
//! Responsibility: collect and build certified API boundary-node state-tree reports.
//! Does not own: CLI parsing, report rendering, or Dashboard aggregate resources.
//! Boundary: performs one response-bounded authenticated `read_state` request per live report.

use super::{
    IcApiBoundaryNodeHostError, IcApiBoundaryNodeReport, IcApiBoundaryNodeRequest,
    IcApiBoundaryNodeRow, IcApiBoundaryNodeSourceData, IcApiBoundaryNodeSourceRequest,
    report_from_source,
};
use crate::{
    agent::build_ic_agent,
    leb128::decode_canonical_unsigned_u64,
    runtime::block_on_current_thread,
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, format_utc_timestamp_secs},
};
use candid::Principal;
use ic_agent::{
    Certificate,
    hash_tree::{HashTree, LookupResult, SubtreeLookupResult},
};
use std::collections::BTreeSet;

const API_BOUNDARY_NODES_LABEL: &[u8] = b"api_boundary_nodes";
const DOMAIN_LABEL: &[u8] = b"domain";
const IPV4_ADDRESS_LABEL: &[u8] = b"ipv4_address";
const IPV6_ADDRESS_LABEL: &[u8] = b"ipv6_address";

///
/// IcApiBoundaryNodeSource
///
/// Source capability for one complete authenticated API boundary-node state tree.
///

pub trait IcApiBoundaryNodeSource {
    /// Fetch and authenticate the complete API boundary-node collection.
    fn fetch_api_boundary_nodes(
        &self,
        request: &IcApiBoundaryNodeSourceRequest,
    ) -> Result<IcApiBoundaryNodeSourceData, IcApiBoundaryNodeHostError>;
}

///
/// LiveIcStateSource
///
/// Built-in live adapter for certified mainnet IC state-tree reports.
///

#[derive(Clone, Copy, Debug, Default)]
pub struct LiveIcStateSource;

impl IcApiBoundaryNodeSource for LiveIcStateSource {
    fn fetch_api_boundary_nodes(
        &self,
        request: &IcApiBoundaryNodeSourceRequest,
    ) -> Result<IcApiBoundaryNodeSourceData, IcApiBoundaryNodeHostError> {
        validate_live_request(request)?;
        let agent = build_ic_agent(&request.endpoint, |reason| {
            IcApiBoundaryNodeHostError::AgentBuild {
                endpoint: request.endpoint.clone(),
                reason,
            }
        })?;
        let effective_canister_id =
            Principal::from_text(&request.effective_canister_id).map_err(|error| {
                IcApiBoundaryNodeHostError::InvalidSourceData {
                    reason: format!("effective_canister_id is invalid: {error}"),
                }
            })?;
        let certificate = block_on_current_thread(agent.read_state_raw(
            vec![vec![API_BOUNDARY_NODES_LABEL.into()]],
            effective_canister_id,
        ))?
        .map_err(|error| IcApiBoundaryNodeHostError::CertifiedReadState {
            endpoint: request.endpoint.clone(),
            reason: error.to_string(),
        })?;
        source_data_from_certificate(request, certificate)
    }
}

/// Build one live complete certified API boundary-node report.
pub fn build_ic_api_boundary_node_report(
    request: &IcApiBoundaryNodeRequest,
) -> Result<IcApiBoundaryNodeReport, IcApiBoundaryNodeHostError> {
    build_ic_api_boundary_node_report_with_source(request, &LiveIcStateSource)
}

/// Build one complete API boundary-node report through a custom authenticated source.
pub fn build_ic_api_boundary_node_report_with_source(
    request: &IcApiBoundaryNodeRequest,
    source: &dyn IcApiBoundaryNodeSource,
) -> Result<IcApiBoundaryNodeReport, IcApiBoundaryNodeHostError> {
    let source_request = source_request(request);
    let source_data = source.fetch_api_boundary_nodes(&source_request)?;
    report_from_source(&source_request, source_data)
}

fn source_request(request: &IcApiBoundaryNodeRequest) -> IcApiBoundaryNodeSourceRequest {
    IcApiBoundaryNodeSourceRequest {
        network: MAINNET_NETWORK.to_string(),
        endpoint: request.source_endpoint.clone(),
        effective_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        observed_at_unix_seconds: request.now_unix_secs,
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        fetched_by: "ic-query".to_string(),
    }
}

fn validate_live_request(
    request: &IcApiBoundaryNodeSourceRequest,
) -> Result<(), IcApiBoundaryNodeHostError> {
    for (field, actual, expected) in [
        ("network", request.network.as_str(), MAINNET_NETWORK),
        (
            "effective_canister_id",
            request.effective_canister_id.as_str(),
            MAINNET_REGISTRY_CANISTER_ID,
        ),
    ] {
        if actual != expected {
            return Err(IcApiBoundaryNodeHostError::InvalidSourceData {
                reason: format!("{field} is {actual:?}, expected {expected:?}"),
            });
        }
    }
    Ok(())
}

pub(super) fn source_data_from_certificate(
    request: &IcApiBoundaryNodeSourceRequest,
    certificate: Certificate,
) -> Result<IcApiBoundaryNodeSourceData, IcApiBoundaryNodeHostError> {
    let certificate_time =
        ic_agent::lookup_value(&certificate, [b"time".as_slice()]).map_err(|error| {
            invalid_certified_state(format!("certificate time is invalid: {error}"))
        })?;
    let certificate_time_unix_nanos =
        decode_canonical_unsigned_u64("certificate time", certificate_time)
            .map_err(invalid_certified_state)?;
    let path = [API_BOUNDARY_NODES_LABEL];
    let tree = match certificate.tree.lookup_subtree(path.iter()) {
        SubtreeLookupResult::Found(tree) => tree,
        SubtreeLookupResult::Absent => {
            return Err(invalid_certified_state(
                "api_boundary_nodes subtree is proven absent",
            ));
        }
        SubtreeLookupResult::Unknown => {
            return Err(invalid_certified_state(
                "api_boundary_nodes subtree is not proven by the certificate",
            ));
        }
    };
    let rows = boundary_node_rows(&tree)?;
    if rows.is_empty() {
        return Err(invalid_certified_state(
            "api_boundary_nodes subtree contains no proven node rows",
        ));
    }
    Ok(IcApiBoundaryNodeSourceData {
        source: request.clone(),
        certificate_time_unix_nanos,
        rows,
    })
}

fn boundary_node_rows(
    tree: &HashTree<Vec<u8>>,
) -> Result<Vec<IcApiBoundaryNodeRow>, IcApiBoundaryNodeHostError> {
    let node_ids = tree
        .list_paths()
        .into_iter()
        .filter_map(|path| path.first().map(|label| label.as_ref().to_vec()))
        .collect::<BTreeSet<_>>();

    node_ids
        .into_iter()
        .map(|node_id| {
            let principal = Principal::try_from_slice(&node_id).map_err(|error| {
                invalid_certified_state(format!(
                    "api_boundary_nodes contains an invalid node principal: {error}"
                ))
            })?;
            Ok(IcApiBoundaryNodeRow {
                node_id: principal.to_text(),
                domain: required_text_leaf(tree, &node_id, DOMAIN_LABEL, "domain")?,
                ipv4_address: optional_text_leaf(
                    tree,
                    &node_id,
                    IPV4_ADDRESS_LABEL,
                    "ipv4_address",
                )?,
                ipv6_address: required_text_leaf(
                    tree,
                    &node_id,
                    IPV6_ADDRESS_LABEL,
                    "ipv6_address",
                )?,
            })
        })
        .collect()
}

fn required_text_leaf(
    tree: &HashTree<Vec<u8>>,
    node_id: &[u8],
    label: &[u8],
    field: &str,
) -> Result<String, IcApiBoundaryNodeHostError> {
    let path = [node_id, label];
    match tree.lookup_path(path.iter()) {
        LookupResult::Found(value) => decode_text_leaf(field, value),
        LookupResult::Absent => Err(invalid_certified_state(format!(
            "API boundary-node {field} leaf is proven absent"
        ))),
        LookupResult::Unknown => Err(invalid_certified_state(format!(
            "API boundary-node {field} leaf is not proven by the certificate"
        ))),
        LookupResult::Error => Err(invalid_certified_state(format!(
            "API boundary-node {field} path does not identify a leaf"
        ))),
    }
}

fn optional_text_leaf(
    tree: &HashTree<Vec<u8>>,
    node_id: &[u8],
    label: &[u8],
    field: &str,
) -> Result<Option<String>, IcApiBoundaryNodeHostError> {
    let path = [node_id, label];
    match tree.lookup_path(path.iter()) {
        LookupResult::Found(value) => decode_text_leaf(field, value).map(Some),
        LookupResult::Absent => Ok(None),
        LookupResult::Unknown => Err(invalid_certified_state(format!(
            "API boundary-node optional {field} leaf is not proven present or absent"
        ))),
        LookupResult::Error => Err(invalid_certified_state(format!(
            "API boundary-node optional {field} path does not identify a leaf"
        ))),
    }
}

fn decode_text_leaf(field: &str, value: &[u8]) -> Result<String, IcApiBoundaryNodeHostError> {
    String::from_utf8(value.to_vec()).map_err(|error| {
        invalid_certified_state(format!(
            "API boundary-node {field} leaf is not UTF-8: {error}"
        ))
    })
}

fn invalid_certified_state(reason: impl Into<String>) -> IcApiBoundaryNodeHostError {
    IcApiBoundaryNodeHostError::InvalidCertifiedState {
        reason: reason.into(),
    }
}
