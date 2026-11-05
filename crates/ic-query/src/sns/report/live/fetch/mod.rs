//! Explicit Governance neuron IO boundary.
mod neurons;
fn governance_canister(
    governance_canister_id: &str,
) -> Result<ic_platform::candid::contract::Principal, super::SnsHostError> {
    super::query::principal_from_text(governance_canister_id, "governance_canister_id")
}
