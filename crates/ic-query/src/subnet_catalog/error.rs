use thiserror::Error as ThisError;

///
/// CatalogError
///
#[derive(Debug, ThisError)]
pub enum CatalogError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("unsupported subnet catalog schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },

    #[error("subnet catalog must contain at least one subnet")]
    EmptySubnets,

    #[error("subnet catalog must contain at least one routing range")]
    EmptyRoutingRanges,

    #[error("invalid principal in {field}: {value}: {reason}")]
    InvalidPrincipal {
        field: &'static str,
        value: String,
        reason: String,
    },

    #[error("duplicate subnet principal in catalog: {subnet_principal}")]
    DuplicateSubnet { subnet_principal: String },

    #[error("routing range references unknown subnet: {subnet_principal}")]
    UnknownRoutingSubnet { subnet_principal: String },

    #[error(
        "invalid routing range for {subnet_principal}: start {start_canister_id} sorts after end {end_canister_id}"
    )]
    InvalidRoutingRange {
        subnet_principal: String,
        start_canister_id: String,
        end_canister_id: String,
    },

    #[error("subnet principal {subnet_principal} was not found in the cached catalog")]
    UnknownSubnet { subnet_principal: String },

    #[error("principal prefix {prefix:?} did not match cached subnet principals")]
    PrincipalPrefixNotFound { prefix: String },

    #[error("principal prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousPrincipalPrefix {
        prefix: String,
        matches: Vec<String>,
    },

    #[error(
        "canister principal {canister_principal} was not covered by cached routing ranges at registry_version={registry_version}, catalog_schema_version={catalog_schema_version}"
    )]
    RouteNotFound {
        canister_principal: String,
        registry_version: u64,
        catalog_schema_version: u32,
    },
}
