//! Module: nns::governance::request
//!
//! Responsibility: describe one direct NNS Governance report collection.
//! Does not own: live transport, report assembly, or persistence policy.
//! Boundary: keeps caller intent separate from source-returned provenance.

use crate::subnet_catalog::format_utc_timestamp_secs;

///
/// NnsGovernanceSourceSelection
///
/// Transport selected for one direct NNS Governance collection.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NnsGovernanceSourceSelection {
    /// Submit an ordinary unreplicated query through one replica endpoint.
    ReplicaQuery {
        /// Replica endpoint selected by the caller.
        endpoint: String,
        /// Collector identity recorded in report provenance.
        fetched_by: String,
    },
    /// Execute a replicated inter-canister call from the current canister.
    ReplicatedInterCanisterCall,
}

///
/// NnsGovernanceRequest
///
/// Network, timestamp, and transport selection for one Governance report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsGovernanceRequest {
    /// Network to query.
    pub network: String,
    /// UTC collection timestamp recorded in the report.
    pub fetched_at: String,
    /// Requested source transport.
    pub source: NnsGovernanceSourceSelection,
}

impl NnsGovernanceRequest {
    /// Create a replica-query request with an explicit UTC timestamp.
    #[must_use]
    pub fn replica_query(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            fetched_at: fetched_at.into(),
            source: NnsGovernanceSourceSelection::ReplicaQuery {
                endpoint: endpoint.into(),
                fetched_by: fetched_by.into(),
            },
        }
    }

    /// Create a replica-query request from a Unix collection timestamp.
    #[must_use]
    pub fn replica_query_from_unix_secs(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at_unix_secs: u64,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self::replica_query(
            network,
            endpoint,
            format_utc_timestamp_secs(fetched_at_unix_secs),
            fetched_by,
        )
    }

    /// Create a replicated inter-canister-call request with an explicit UTC timestamp.
    #[must_use]
    pub fn replicated_inter_canister_call(
        network: impl Into<String>,
        fetched_at: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            fetched_at: fetched_at.into(),
            source: NnsGovernanceSourceSelection::ReplicatedInterCanisterCall,
        }
    }

    /// Create a replicated inter-canister-call request from a Unix collection timestamp.
    #[must_use]
    pub fn replicated_inter_canister_call_from_unix_secs(
        network: impl Into<String>,
        fetched_at_unix_secs: u64,
    ) -> Self {
        Self::replicated_inter_canister_call(
            network,
            format_utc_timestamp_secs(fetched_at_unix_secs),
        )
    }
}
