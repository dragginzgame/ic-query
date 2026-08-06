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
    pub(crate) chunk_value_bytes: usize,
    pub(crate) value_bytes: usize,
    pub(crate) chunk_reference_count: usize,
    pub(crate) chunk_evidence_bytes: usize,
    pub(crate) chunk_evidence: Vec<CertifiedRegistryChunkEvidence>,
    pub(crate) chunk_query_call_count: usize,
    pub(crate) chunk_response_bytes: usize,
    pub(crate) more_available: bool,
    pub(crate) certified_response_bytes: usize,
    pub(crate) certificate_time_nanos: u64,
    pub(crate) root_key_digest: String,
    pub(crate) certificate_hex: String,
    pub(crate) certificate_bytes: usize,
    pub(crate) hash_tree_hex: String,
    pub(crate) hash_tree_bytes: usize,
}

///
/// AuthenticatedRegistryDeltaWitness
///
/// Registry delta contents and provenance decoded directly from an authenticated witness.
///

#[derive(Debug)]
pub struct AuthenticatedRegistryDeltaWitness {
    pub(crate) certified_latest_version: u64,
    pub(crate) versions: Vec<CertifiedRegistryDeltaVersion>,
    pub(crate) mutation_count: usize,
    pub(crate) precondition_count: usize,
    pub(crate) inline_value_bytes: usize,
    pub(crate) chunk_reference_count: usize,
    pub(crate) more_available: bool,
    pub(crate) certificate_time_nanos: u64,
    pub(crate) root_key_digest: String,
    pub(crate) certificate_hex: String,
    pub(crate) certificate_bytes: usize,
    pub(crate) hash_tree_hex: String,
    pub(crate) hash_tree_bytes: usize,
}

///
/// CertifiedRegistryChunkEvidence
///
/// One unique hash-verified Registry chunk retained as complete batch evidence.
///

#[derive(Debug)]
pub struct CertifiedRegistryChunkEvidence {
    pub(crate) sha256: [u8; 32],
    pub(crate) content: Vec<u8>,
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
/// One ordered Registry mutation with raw type, original encoding, and complete content.
///

#[derive(Debug)]
pub struct CertifiedRegistryMutation {
    pub(crate) mutation_type: i32,
    pub(crate) key_hex: String,
    pub(crate) value_hex: Option<String>,
    pub(crate) value_encoding: CertifiedRegistryValueEncoding,
    pub(crate) chunk_sha256s: Vec<Vec<u8>>,
}

///
/// CertifiedRegistryValueEncoding
///
/// Original value representation committed by one certified Registry mutation.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertifiedRegistryValueEncoding {
    Absent,
    Inline,
    Chunked,
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
