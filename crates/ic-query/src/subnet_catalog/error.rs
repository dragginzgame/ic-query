use crate::subnet_catalog::RoutingRange;
use thiserror::Error as ThisError;

///
/// CatalogError
///
/// Errors returned while parsing, validating, or resolving a subnet catalog.
///

#[derive(Debug, ThisError)]
pub enum CatalogError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("unsupported subnet catalog schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },

    /// Catalog network identity does not match the required authority.
    #[error("subnet catalog network is {actual:?}; expected {expected:?}")]
    NetworkMismatch {
        /// Required network identity.
        expected: String,
        /// Catalog network identity.
        actual: String,
    },

    /// Catalog Registry canister identity does not match mainnet authority.
    #[error("subnet catalog Registry canister is {actual}; expected {expected}")]
    RegistryCanisterMismatch {
        /// Required Registry canister principal.
        expected: String,
        /// Catalog Registry canister principal.
        actual: String,
    },

    /// Registry version zero cannot identify an authority snapshot.
    #[error("subnet catalog Registry version must be greater than zero")]
    InvalidRegistryVersion,

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

    /// Subnet rows are not strictly ordered by canonical principal text.
    #[error("noncanonical subnet order: {previous} must sort before {current}")]
    NonCanonicalSubnetOrder {
        /// Previous Subnet principal.
        previous: String,
        /// Current Subnet principal.
        current: String,
    },

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

    #[error("overlapping routing ranges: {first} overlaps {second}")]
    OverlappingRoutingRanges {
        first: Box<RoutingRange>,
        second: Box<RoutingRange>,
    },

    /// Routing ranges are not strictly ordered by principal bytes and target.
    #[error("noncanonical routing range order: {previous} must sort before {current}")]
    NonCanonicalRoutingOrder {
        /// Previous routing range.
        previous: Box<RoutingRange>,
        /// Current routing range.
        current: Box<RoutingRange>,
    },

    /// Derived Subnet kind contradicts its raw Registry numeric code.
    #[error(
        "subnet {subnet_principal} kind {actual} contradicts Registry subnet_type={registry_subnet_type}; expected {expected}"
    )]
    SubnetKindMismatch {
        /// Subnet principal.
        subnet_principal: String,
        /// Raw Registry numeric discriminant.
        registry_subnet_type: i32,
        /// Kind required by the raw discriminant.
        expected: String,
        /// Kind supplied by the raw catalog.
        actual: String,
    },

    /// Charging default contradicts the raw Registry Subnet kind.
    #[error(
        "subnet {subnet_principal} charges_apply_by_default={actual} contradicts expected {expected}"
    )]
    ChargingPolicyMismatch {
        /// Subnet principal.
        subnet_principal: String,
        /// Charging default required by the raw kind.
        expected: bool,
        /// Charging default supplied by the raw catalog.
        actual: bool,
    },

    /// Classification metadata contradicts its policy or source.
    #[error("subnet {subnet_principal} classification field {field} is invalid: {reason}")]
    ClassificationMismatch {
        /// Subnet principal.
        subnet_principal: String,
        /// Invalid classification field.
        field: &'static str,
        /// Deterministic mismatch reason.
        reason: String,
    },

    /// A provenance timestamp is not valid canonical UTC text.
    #[error("invalid catalog timestamp in {field}: {value:?}")]
    InvalidTimestamp {
        /// Timestamp field.
        field: &'static str,
        /// Invalid timestamp text.
        value: String,
    },

    /// A provenance timestamp is beyond the caller's future-skew policy.
    #[error(
        "catalog timestamp in {field} is in the future: {value}; latest accepted unix time is {latest_allowed_unix_secs}"
    )]
    FutureTimestamp {
        /// Timestamp field.
        field: &'static str,
        /// Future timestamp text.
        value: String,
        /// Latest accepted Unix time.
        latest_allowed_unix_secs: u64,
    },

    /// A source endpoint is not a clean credential-free HTTP(S) base URL.
    #[error("invalid catalog source endpoint {endpoint:?}: {reason}")]
    InvalidSourceEndpoint {
        /// Invalid endpoint.
        endpoint: String,
        /// Endpoint validation reason.
        reason: String,
    },

    /// Provenance evidence is internally inconsistent.
    #[error("invalid catalog provenance field {field}: {reason}")]
    InvalidProvenance {
        /// Invalid provenance field.
        field: &'static str,
        /// Deterministic validation reason.
        reason: String,
    },

    /// The build does not yet have a verifier for the claimed assurance level.
    #[error(
        "unsupported catalog assurance {assurance}; this build validates uncertified_query and multi_endpoint_agreement evidence"
    )]
    UnsupportedAssurance {
        /// Unsupported assurance label.
        assurance: String,
    },

    /// Classification policy schema is not supported by this build.
    #[error("classification policy schema {found} is unsupported; expected {supported}")]
    ClassificationPolicyVersionMismatch {
        /// Catalog policy schema.
        found: u32,
        /// Supported policy schema.
        supported: u32,
    },

    /// Classification policy digest does not match this build.
    #[error("classification policy digest mismatch: found {actual}; expected {expected}")]
    ClassificationPolicyDigestMismatch {
        /// Expected policy digest.
        expected: String,
        /// Catalog policy digest.
        actual: String,
    },

    /// Resolver implementation identity does not match this build.
    #[error(
        "resolver policy mismatch: version {actual_version} backend {actual_backend:?}; expected version {expected_version} backend {expected_backend:?}"
    )]
    ResolverPolicyMismatch {
        /// Expected resolver schema.
        expected_version: u32,
        /// Catalog resolver schema.
        actual_version: u32,
        /// Expected resolver backend.
        expected_backend: String,
        /// Catalog resolver backend.
        actual_backend: String,
    },

    /// Catalog digest is not exactly 32 lowercase hexadecimal bytes.
    #[error("invalid catalog digest {value:?}; expected 64 lowercase hexadecimal characters")]
    InvalidCatalogDigest {
        /// Invalid digest text.
        value: String,
    },

    /// Agreement digest is absent or not exactly 32 lowercase hexadecimal bytes.
    #[error(
        "invalid catalog agreement digest {value:?}; expected 64 lowercase hexadecimal characters"
    )]
    InvalidAgreementDigest {
        /// Invalid digest text.
        value: String,
    },

    /// Stored agreement digest does not match the canonical Registry payload.
    #[error("catalog agreement digest mismatch: found {actual}; expected {expected}")]
    AgreementDigestMismatch {
        /// Recomputed agreement digest.
        expected: String,
        /// Stored agreement digest.
        actual: String,
    },

    /// Stored catalog digest does not match the canonical authority payload.
    #[error("catalog digest mismatch: found {actual}; expected {expected}")]
    CatalogDigestMismatch {
        /// Recomputed digest.
        expected: String,
        /// Stored digest.
        actual: String,
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
