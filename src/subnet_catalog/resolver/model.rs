use crate::subnet_catalog::{RoutingRange, SubnetInfo};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

///
/// ResolveAs
///
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolveAs {
    Subnet,
    Canister,
}

impl ResolveAs {
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
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedSubnetSubject {
    Subnet,
    Canister,
}

impl ResolvedSubnetSubject {
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolvedSubnet {
    pub input_principal: String,
    pub resolved_as: ResolvedSubnetSubject,
    pub resolved_from: String,
    pub subnet: SubnetInfo,
    pub matched_canister_principal: Option<String>,
    pub matched_routing_range: Option<RoutingRange>,
}
