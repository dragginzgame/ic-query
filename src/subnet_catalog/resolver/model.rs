//! Module: subnet_catalog::resolver::model
//!
//! Defines resolver options and outputs for subnet catalog lookups.

use crate::subnet_catalog::{RoutingRange, SubnetInfo};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

///
/// ResolveAs
///
/// Caller-requested interpretation for an ambiguous principal input.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolveAs {
    /// Resolve the input as a subnet principal.
    Subnet,
    /// Resolve the input as a canister principal.
    Canister,
}

impl ResolveAs {
    /// Returns the stable snake_case value used in CLI options and reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Subnet => "subnet",
            Self::Canister => "canister",
        }
    }
}

impl FromStr for ResolveAs {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "subnet" => Ok(Self::Subnet),
            "canister" => Ok(Self::Canister),
            other => Err(format!("invalid value {other}; use subnet or canister")),
        }
    }
}

///
/// ResolvedSubnetSubject
///
/// Subject type chosen by the resolver.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedSubnetSubject {
    /// Resolver matched a subnet principal.
    Subnet,
    /// Resolver matched a canister principal through routing ranges.
    Canister,
}

impl ResolvedSubnetSubject {
    /// Returns the stable snake_case value used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Subnet => "subnet",
            Self::Canister => "canister",
        }
    }
}

///
/// ResolvedSubnet
///
/// Resolved subnet match and the evidence used to produce it.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolvedSubnet {
    pub input_principal: String,
    pub resolved_as: ResolvedSubnetSubject,
    pub resolved_from: String,
    pub subnet: SubnetInfo,
    pub matched_canister_principal: Option<String>,
    pub matched_routing_range: Option<RoutingRange>,
}
