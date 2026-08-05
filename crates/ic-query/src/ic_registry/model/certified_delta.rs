///
/// CertifiedRegistryDeltaBatch
///
/// Authenticated and structurally validated contents of one Registry delta query.
///

#[derive(Debug)]
pub struct CertifiedRegistryDeltaBatch {
    pub(crate) requested_version: u64,
    pub(crate) certified_latest_version: u64,
    pub(crate) versions: Vec<CertifiedRegistryDeltaVersion>,
    pub(crate) mutation_count: usize,
    pub(crate) precondition_count: usize,
    pub(crate) inline_value_bytes: usize,
    pub(crate) more_available: bool,
    pub(crate) response_bytes: usize,
    pub(crate) certificate_time_nanos: u64,
    pub(crate) root_key_digest: String,
    pub(crate) certificate_hex: String,
    pub(crate) certificate_bytes: usize,
    pub(crate) hash_tree_hex: String,
    pub(crate) hash_tree_bytes: usize,
}

///
/// CertifiedRegistryDeltaVersion
///
/// One contiguous Registry version decoded from the certified delta tree.
///

#[derive(Debug)]
pub struct CertifiedRegistryDeltaVersion {
    pub(crate) version: u64,
    pub(crate) timestamp_nanoseconds: u64,
    pub(crate) mutations: Vec<CertifiedRegistryMutation>,
    pub(crate) preconditions: Vec<CertifiedRegistryPrecondition>,
}

///
/// CertifiedRegistryMutation
///
/// One ordered Registry mutation with its raw type and complete inline content.
///

#[derive(Debug)]
pub struct CertifiedRegistryMutation {
    pub(crate) mutation_type: i32,
    pub(crate) key_hex: String,
    pub(crate) value_hex: Option<String>,
}

///
/// CertifiedRegistryPrecondition
///
/// One key-version precondition attached to an atomic Registry mutation.
///

#[derive(Debug)]
pub struct CertifiedRegistryPrecondition {
    pub(crate) key_hex: String,
    pub(crate) expected_version: u64,
}
