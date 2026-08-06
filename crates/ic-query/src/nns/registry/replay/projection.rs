//! Module: nns::registry::replay::projection
//!
//! Responsibility: project complete replay state through the shared Subnet Catalog content path.
//! Does not own: source authentication, serialization, cache policy, or assurance promotion.
//! Boundary: the projection borrows its complete session and cannot outlive its replay evidence.

use super::NnsRegistryReplaySession;
use crate::{
    ic_registry::{
        ROUTING_TABLE_KEY, SUBNET_LIST_KEY, proto::RoutingTable, proto::SubnetListRecord,
        proto::SubnetRecord, routing_ranges_from_table, subnet_info_from_record, subnet_record_key,
    },
    subnet_catalog::{CatalogError, RoutingRange, SubnetInfo, canonicalize_subnet_catalog_content},
};
use candid::Principal;
use prost::Message;
use thiserror::Error as ThisError;

///
/// NnsRegistrySubnetCatalogProjection
///
/// Canonical Subnet Catalog rows derived from one complete exact-target replay session.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsRegistrySubnetCatalogProjection<'a> {
    session: &'a NnsRegistryReplaySession,
    registry_version: u64,
    subnets: Vec<SubnetInfo>,
    routing_ranges: Vec<RoutingRange>,
}

impl<'a> NnsRegistrySubnetCatalogProjection<'a> {
    /// Return the complete replay session that qualifies this projection.
    #[must_use]
    pub const fn replay_session(&self) -> &'a NnsRegistryReplaySession {
        self.session
    }

    /// Return the exact Registry version shared by every projected record.
    #[must_use]
    pub const fn registry_version(&self) -> u64 {
        self.registry_version
    }

    /// Return canonical Subnet rows classified through the existing catalog policy.
    #[must_use]
    pub fn subnets(&self) -> &[SubnetInfo] {
        &self.subnets
    }

    /// Return canonical inclusive canister routing ranges.
    #[must_use]
    pub fn routing_ranges(&self) -> &[RoutingRange] {
        &self.routing_ranges
    }
}

///
/// NnsRegistrySubnetCatalogProjectionError
///
/// Typed failures returned before replay state is exposed as catalog content.
///

#[derive(Debug, ThisError)]
pub enum NnsRegistrySubnetCatalogProjectionError {
    /// The replay session has not reached its pinned exact target.
    #[error(
        "Registry replay session is incomplete: selected version {selected_version:?}, through version {through_version}"
    )]
    IncompleteSession {
        /// Exact target selected from the first admitted report, when available.
        selected_version: Option<u64>,
        /// Last Registry version currently reconstructed.
        through_version: u64,
    },

    /// Registry version zero cannot identify a catalog snapshot.
    #[error("Registry replay selected version must be greater than zero for catalog projection")]
    InvalidRegistryVersion,

    /// Complete replay state does not contain one record required by the catalog.
    #[error("complete Registry replay state is missing required key {key:?}")]
    MissingRequiredRegistryKey {
        /// Exact raw Registry key interpreted as canonical UTF-8 text.
        key: String,
    },

    /// One required replayed Registry record could not be interpreted.
    #[error("replayed Registry key {key:?} is not a valid {message}: {reason}")]
    InvalidRegistryRecord {
        /// Exact Registry key containing the invalid value.
        key: String,
        /// Expected Registry record type.
        message: &'static str,
        /// Deterministic decoding or structural failure.
        reason: String,
    },

    /// Canonical catalog classification or routing validation failed.
    #[error(transparent)]
    Catalog(#[from] CatalogError),
}

/// Project a complete exact-target replay session into canonical Subnet Catalog content.
///
/// The returned value borrows `session`, keeping the projected rows attached to
/// their replay provenance. It is not a serialized mirror, a
/// `ValidatedSubnetCatalog`, or a `CatalogAssurance::Certified` promotion.
pub fn project_nns_registry_subnet_catalog(
    session: &NnsRegistryReplaySession,
) -> Result<NnsRegistrySubnetCatalogProjection<'_>, NnsRegistrySubnetCatalogProjectionError> {
    let selected_version = session.selected_version();
    let (true, Some(registry_version)) = (
        session.is_complete() && session.complete_state_digest().is_some(),
        selected_version,
    ) else {
        return Err(NnsRegistrySubnetCatalogProjectionError::IncompleteSession {
            selected_version,
            through_version: session.state().through_version(),
        });
    };
    if registry_version == 0 {
        return Err(NnsRegistrySubnetCatalogProjectionError::InvalidRegistryVersion);
    }

    let state = session.state();
    let subnet_list =
        decode_required_record::<SubnetListRecord>(state, SUBNET_LIST_KEY, "SubnetListRecord")?;
    let routing_table =
        decode_required_record::<RoutingTable>(state, ROUTING_TABLE_KEY, "RoutingTable")?;
    let mut subnets = Vec::with_capacity(subnet_list.subnets.len());
    for raw_subnet_principal in subnet_list.subnets {
        let subnet_principal = Principal::try_from_slice(&raw_subnet_principal)
            .map(|principal| principal.to_text())
            .map_err(|error| invalid_record(SUBNET_LIST_KEY, "SubnetListRecord", error))?;
        let record_key = subnet_record_key(&subnet_principal);
        let record = decode_required_record::<SubnetRecord>(state, &record_key, "SubnetRecord")?;
        subnets.push(subnet_info_from_record(&subnet_principal, &record));
    }
    let mut routing_ranges = routing_ranges_from_table(&routing_table)
        .map_err(|error| invalid_record(ROUTING_TABLE_KEY, "RoutingTable", error))?;
    canonicalize_subnet_catalog_content(&mut subnets, &mut routing_ranges)?;

    Ok(NnsRegistrySubnetCatalogProjection {
        session,
        registry_version,
        subnets,
        routing_ranges,
    })
}

fn decode_required_record<M>(
    state: &super::NnsRegistryReplayState,
    key: &str,
    message: &'static str,
) -> Result<M, NnsRegistrySubnetCatalogProjectionError>
where
    M: Message + Default,
{
    let value = state.get(key.as_bytes()).ok_or_else(|| {
        NnsRegistrySubnetCatalogProjectionError::MissingRequiredRegistryKey {
            key: key.to_string(),
        }
    })?;
    M::decode(value.value()).map_err(|error| invalid_record(key, message, error))
}

fn invalid_record(
    key: &str,
    message: &'static str,
    error: impl ToString,
) -> NnsRegistrySubnetCatalogProjectionError {
    NnsRegistrySubnetCatalogProjectionError::InvalidRegistryRecord {
        key: key.to_string(),
        message,
        reason: error.to_string(),
    }
}
