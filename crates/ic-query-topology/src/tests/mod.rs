//! Replay-only Query test data; regeneration belongs to Platform qualification.

use ic_platform_protocol::candid::Nat;
use ic_platform_types::Principal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubnetFrame {
    pub replica_version: String,
    pub maximum_state_size: Option<u64>,
    pub halted: bool,
    pub node_ids: Vec<Principal>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullViewFrame {
    pub id: String,
    pub subnet_id: Option<Principal>,
    pub engine_operator_id: Option<Principal>,
    pub subnet: Option<SubnetFrame>,
    pub deleted_at: Option<i64>,
    pub node_ids: Vec<Principal>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NodeClass {
    #[serde(rename = "type4")]
    Type4,
    #[serde(rename = "type4_1")]
    Type4_1,
    #[serde(rename = "type4_2")]
    Type4_2,
    #[serde(rename = "type4_3")]
    Type4_3,
    #[serde(rename = "type4_4")]
    Type4_4,
    #[serde(rename = "type4_5")]
    Type4_5,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Price {
    pub gross_cycles: Nat,
    pub net_cycles: Nat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceEntry {
    pub dc: Option<String>,
    pub updated_at: i64,
    pub node_type: NodeClass,
    pub provider: Option<Principal>,
    pub price: Price,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketplaceEntry {
    pub key: String,
    pub entry: PriceEntry,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AvailableNode {
    pub dc: Option<String>,
    pub node_type: NodeClass,
    pub node_id: Principal,
    pub provider: Principal,
    pub operator: Principal,
    pub sev_enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Page {
    pub next: Option<Principal>,
    pub registry_version: u64,
    pub nodes: Vec<AvailableNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EngineError {
    Internal(String),
    ControllerError(String),
    InvalidTransition(String),
    SlugConflict(String),
    NotFound,
    Unauthorized,
    AlreadyExists,
    NotVetted,
    BadRequest(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PageResult {
    #[serde(rename = "ok")]
    Ok(Option<Page>),
    #[serde(rename = "err")]
    Err(EngineError),
}

#[derive(Serialize)]
pub struct RegistryCopyStatus {
    pub version: u64,
    pub latest: u64,
    pub caught_up: bool,
}

#[derive(Serialize)]
pub struct Placement {
    pub subnet_id: Option<Principal>,
}

#[derive(Deserialize)]
struct Corpus {
    version: u32,
    frames: Vec<Frame>,
}

#[derive(Deserialize)]
struct Frame {
    schema: String,
    input: serde_json::Value,
    hex: String,
}

/// Look up an independently generated frame. Missing inputs fail explicitly:
/// new test cases require reviewed corpus regeneration at the supplying owner.
pub fn frame(schema: &str, input: &serde_json::Value) -> Vec<u8> {
    static CORPUS: std::sync::OnceLock<Corpus> = std::sync::OnceLock::new();
    let corpus = CORPUS.get_or_init(|| {
        serde_json::from_str(include_str!("corpus-v1.json")).expect("valid frozen corpus")
    });
    assert_eq!(corpus.version, 1);
    let value = corpus
        .frames
        .iter()
        .find(|entry| entry.schema == schema && &entry.input == input)
        .unwrap_or_else(|| panic!("missing independent {schema} fixture; regenerate the corpus"));
    assert_eq!(value.hex.len() % 2, 0);

    value
        .hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

pub trait Fixture: Serialize + 'static {
    const SCHEMA: &'static str;
    fn input(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

impl Fixture for Vec<FullViewFrame> {
    const SCHEMA: &'static str = "Vec<FullViewFrame>";
}

impl Fixture for Vec<MarketplaceEntry> {
    const SCHEMA: &'static str = "Vec<MarketplaceEntry>";
}

impl Fixture for PageResult {
    const SCHEMA: &'static str = "PageResult";
}

impl Fixture for RegistryCopyStatus {
    const SCHEMA: &'static str = "RegistryCopyStatus";
}

impl Fixture for Result<Placement, String> {
    const SCHEMA: &'static str = "Result<Placement, String>";
}

impl Fixture for f64 {
    const SCHEMA: &'static str = "f64";
    fn input(&self) -> serde_json::Value {
        self.to_bits().to_string().into()
    }
}

pub fn encode_fixture<T: Fixture>(value: &T, maximum: usize) -> Result<Vec<u8>, &'static str> {
    let bytes = frame(T::SCHEMA, &value.input());
    if bytes.len() > maximum {
        return Err("fixture output bound");
    }

    Ok(bytes)
}
