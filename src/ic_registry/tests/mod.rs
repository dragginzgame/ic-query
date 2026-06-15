use super::*;
use crate::subnet_catalog::{CATALOG_SCHEMA_VERSION, SubnetKind, SubnetSpecialization};
use proto::{
    PrincipalId, RegistryErrorCode, RegistryGetValueResponse, registry_get_value_response,
};

mod catalog;
mod fixtures;
mod governance;
mod inventory;
mod wire;
