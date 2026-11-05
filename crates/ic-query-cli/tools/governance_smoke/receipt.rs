//! Typed attempt evidence; external build and report payloads retain their JSON shape.

use serde::{Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;

///
/// Status
///
/// Overall attempt outcome, independent of network cleanup.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Running,
    Failed,
    Passed,
}

///
/// NetworkCleanup
///
/// Cleanup evidence for the runtime owned by this attempt.
///

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkCleanup {
    Pending,
    Stopped,
    Failed,
}

///
/// ReportKind
///
/// Supported Governance probe reports and their receipt keys.
///

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportKind {
    Economics,
    MaturityModulation,
    Metrics,
    RewardEvent,
}

impl ReportKind {
    pub const ALL: [Self; 4] = [
        Self::Economics,
        Self::Metrics,
        Self::RewardEvent,
        Self::MaturityModulation,
    ];
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Economics => "economics",
            Self::Metrics => "metrics",
            Self::RewardEvent => "reward_event",
            Self::MaturityModulation => "maturity_modulation",
        }
    }
}

///
/// Phase
///
/// Durable lifecycle boundary, including individual report admission.
///

#[derive(Clone, Copy)]
pub enum Phase {
    Starting,
    CheckingNetwork,
    StartingNetwork,
    ReadingNetwork,
    Deploying,
    Building,
    CheckingModule,
    CheckingProtocol,
    CheckingBuildMetadata,
    Collecting(ReportKind),
    Validating(ReportKind),
    Collected(ReportKind),
    CheckingFinalModule,
    StoppingNetwork,
    Finished,
}

impl Serialize for Phase {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let name = match self {
            Self::Starting => "starting",
            Self::CheckingNetwork => "checking_network",
            Self::StartingNetwork => "starting_network",
            Self::ReadingNetwork => "reading_network",
            Self::Deploying => "deploying",
            Self::Building => "building",
            Self::CheckingModule => "checking_module",
            Self::CheckingProtocol => "checking_protocol",
            Self::CheckingBuildMetadata => "checking_build_metadata",
            Self::CheckingFinalModule => "checking_final_module",
            Self::StoppingNetwork => "stopping_network",
            Self::Finished => "finished",
            Self::Collecting(kind) => {
                return serializer.serialize_str(&format!("collecting_{}", kind.as_str()));
            }
            Self::Validating(kind) => {
                return serializer.serialize_str(&format!("validating_{}", kind.as_str()));
            }
            Self::Collected(kind) => {
                return serializer.serialize_str(&format!("collected_{}", kind.as_str()));
            }
        };
        serializer.serialize_str(name)
    }
}

///
/// ReportReceipt
///
/// Reply destination is published before its decoded report is admitted.
///

#[derive(Serialize)]
pub struct ReportReceipt {
    pub reply_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<Value>,
}

///
/// Receipt
///
/// Attempt lifecycle and retained evidence, serialized with the schema-1 field contract.
///

#[derive(Serialize)]
pub struct Receipt {
    pub schema_version: u32,
    pub status: Status,
    pub environment: String,
    pub evidence_scope: String,
    pub icp: String,
    pub rustc: String,
    pub started_at: String,
    pub point_in_time_guaranteed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<Phase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_phase: Option<Phase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // Retain malformed discovery values as evidence before rejecting them.
    pub api_endpoint: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canister_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_hash_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_hash_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reports: Option<BTreeMap<ReportKind, ReportReceipt>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_cleanup: Option<NetworkCleanup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupted_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_cleanup_errors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_error: Option<String>,
    #[serde(rename = "receipt_errors", skip_serializing_if = "Option::is_none")]
    pub storage_errors: Option<Vec<String>>,
}

impl Receipt {
    pub fn new(environment: &str, icp: String, rustc: String, started_at: String) -> Self {
        Self {
            schema_version: 1,
            status: Status::Running,
            environment: environment.to_string(),
            evidence_scope: if environment == "local" {
                "local_nns"
            } else {
                "mainnet"
            }
            .to_string(),
            icp,
            rustc,
            started_at,
            point_in_time_guaranteed: false,
            phase: None,
            failed_phase: None,
            updated_at: None,
            finished_at: None,
            network: None,
            api_endpoint: None,
            canister_id: None,
            module_hash_before: None,
            module_hash_after: None,
            protocol_sha256: None,
            build: None,
            reports: None,
            network_cleanup: None,
            error: None,
            interrupted_by: None,
            command_stdout: None,
            command_stderr: None,
            command_cleanup_errors: None,
            cleanup_error: None,
            storage_errors: None,
        }
    }
}
