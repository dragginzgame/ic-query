//! Explicit target for retained neuron operations.
use crate::sns::report::SnsHostError;

///
/// SnsNeuronTargetRequest
///
/// Network, endpoint, collection time and explicit Governance principal.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronTargetRequest {
    /// Selected network.
    pub network: String,
    /// Explicit replica endpoint.
    pub source_endpoint: String,
    /// Caller acquisition clock.
    pub now_unix_secs: u64,
    /// Governance canister holding the neurons.
    pub governance_canister_id: String,
}
impl SnsNeuronTargetRequest {
    /// Construct a target without SNS-W, Root, metadata or Swap calls.
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        governance_canister_id: impl Into<String>,
    ) -> Result<Self, SnsHostError> {
        let governance_canister_id = governance_canister_id.into();
        crate::sns::report::target::parse_governance_canister_input(&governance_canister_id)?;
        Ok(Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            governance_canister_id,
        })
    }
}
