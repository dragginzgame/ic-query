//! Exact neuron stake and principal-permission evidence.
use serde::{Deserialize, Serialize};

///
/// SnsNeuronPermissionRow
///
/// Canonical principal and the native permission codes it holds on a neuron.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronPermissionRow {
    /// Principal holding these permissions; not an eligibility verdict.
    pub principal: String,
    /// Native permission codes, retaining unknown codes.
    pub permission_types: Vec<i32>,
}

///
/// SnsNeuronDetailReport
///
/// Exact neuron identity, stake and permissions from an explicit Governance target.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnsNeuronDetailReport {
    /// Current report schema, always 1 before 1.0.
    pub schema_version: u32,
    /// Explicit target and acquisition evidence, flattened in JSON.
    #[serde(flatten)]
    pub context: crate::sns::SnsReportContext,
    /// Requested 32-byte neuron id as lowercase hexadecimal text.
    pub neuron_id: String,
    /// Collection origin.
    pub data_source: crate::report::ReportDataSource,
    /// Raw stake, dissolve and permission evidence.
    pub neuron: super::SnsNeuronRow,
}

/// Return the label for a native permission code, retaining unknown numeric codes separately.
#[must_use]
pub const fn sns_neuron_permission_name(code: i32) -> &'static str {
    match code {
        0 => "unspecified",
        1 => "configure_dissolve_state",
        2 => "manage_principals",
        3 => "submit_proposal",
        4 => "vote",
        5 => "disburse",
        6 => "split",
        7 => "merge_maturity",
        8 => "disburse_maturity",
        9 => "stake_maturity",
        10 => "manage_voting_permission",
        _ => "unknown",
    }
}
