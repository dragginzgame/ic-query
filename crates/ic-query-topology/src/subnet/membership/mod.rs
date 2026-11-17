//! Compare registered members and observed node keys without claiming an atomic snapshot.

use ic_platform_types::Principal;
use serde::Serialize;
use thiserror::Error;

use super::configuration::{self, SubnetConfigurationError, SubnetConfigurationReport};
use super::{SubnetError, SubnetReport};

/// Two separately acquired reports and their node-set differences.
#[derive(Debug, Serialize)]
pub struct SubnetMembershipReport {
    /// Query-owned report schema; remains 1 before 1.0.
    pub schema: u8,
    /// False: the reports have no shared version or certified observation time.
    pub atomic_snapshot: bool,
    /// First read: Registry configuration, retaining its query-signature evidence.
    pub configuration: SubnetConfigurationReport,
    /// Second read: subnet-certified node-key observation, with incomplete coverage.
    pub observation: SubnetReport,
    /// Registered IDs absent from the returned key observation; not failed nodes.
    pub registered_without_observed_key: Vec<Principal>,
    /// Observed key IDs absent from the returned record; not unauthorized nodes.
    pub observed_key_without_registration: Vec<Principal>,
}

/// No failed comparison yields a successful membership or health claim.
#[derive(Debug, Error)]
pub enum SubnetMembershipError {
    /// Registry configuration failed; the subnet read was not started.
    #[error("membership configuration read failed: {0}")]
    Configuration(#[source] SubnetConfigurationError),
    /// Subnet observation failed after successful configuration acquisition.
    #[error("membership node-key observation failed after retaining configuration: {source}")]
    Observation {
        /// Retained first report, not a completed comparison.
        configuration: Box<SubnetConfigurationReport>,
        /// Original typed subnet read failure.
        #[source]
        source: SubnetError,
    },
    /// Refuse to compare reports for any subnet other than the selected one.
    #[error(
        "membership sources identify subnets {configured} and {observed}, expected {requested}"
    )]
    IdentityMismatch {
        /// Explicitly selected subnet.
        requested: Principal,
        /// Subnet identified by the Registry report's queried key.
        configured: Principal,
        /// Subnet identified by the certified observation.
        observed: Principal,
    },
}

/// Compare one subnet's registered members and certified node-key observation.
///
/// Reads configuration and then subnet state using their existing bounded readers.
/// Each permits at most 1024 IDs and a 20-second asynchronous deadline. Requires
/// Tokio IO and time; no tasks, retries or persistent history are introduced.
/// Matching sets do not prove live nodes or an atomic Registry/state snapshot.
///
/// # Errors
/// Retains typed read causes and configuration on a second-read failure. Refuses
/// mismatched subnet identities before comparing the admitted sorted lists.
pub async fn fetch(subnet_id: Principal) -> Result<SubnetMembershipReport, SubnetMembershipError> {
    collect(
        subnet_id,
        configuration::fetch(subnet_id),
        super::fetch(subnet_id),
    )
    .await
}

async fn collect(
    subnet_id: Principal,
    configuration: impl Future<Output = Result<SubnetConfigurationReport, SubnetConfigurationError>>,
    observation: impl Future<Output = Result<SubnetReport, SubnetError>>,
) -> Result<SubnetMembershipReport, SubnetMembershipError> {
    let configuration = configuration
        .await
        .map_err(SubnetMembershipError::Configuration)?;
    let observation = match observation.await {
        Ok(report) => report,
        Err(source) => {
            return Err(SubnetMembershipError::Observation {
                configuration: Box::new(configuration),
                source,
            });
        }
    };
    if configuration.subnet_id != subnet_id || observation.subnet_id != subnet_id {
        return Err(SubnetMembershipError::IdentityMismatch {
            requested: subnet_id,
            configured: configuration.subnet_id,
            observed: observation.subnet_id,
        });
    }
    // Both private production inputs come from readers that sort, bound and
    // reject duplicate IDs. Do not weaken or repeat their admission here.
    let registered_without_observed_key = configuration
        .registered_node_ids
        .iter()
        .filter(|id| observation.observed_node_ids.binary_search(id).is_err())
        .copied()
        .collect();
    let observed_key_without_registration = observation
        .observed_node_ids
        .iter()
        .filter(|id| configuration.registered_node_ids.binary_search(id).is_err())
        .copied()
        .collect();
    Ok(SubnetMembershipReport {
        schema: 1,
        atomic_snapshot: false,
        configuration,
        observation,
        registered_without_observed_key,
        observed_key_without_registration,
    })
}

#[cfg(test)]
mod tests;
