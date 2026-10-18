//! Narrow current ICP block projection from ic_ledger/pb/v1/types.proto.
//! Schema: <https://github.com/dfinity/ic/blob/ef2369f7abab66b2254995e73355002a8130987f/rs/ledger_suite/icp/proto/ic_ledger/pb/v1/types.proto>.
//!
//! Unknown operations, extensions and fields fail the owning decoder's exact
//! re-encoding check. These types do not reproduce Ledger state or defaults.

///
/// Block
///
/// Narrow block projection for canonical transfer admission.
///

#[derive(Clone, prost::Message, PartialEq)]
pub(super) struct Block {
    #[prost(message, optional, tag = "1")]
    pub parent_hash: Option<BytesValue>,
    #[prost(message, optional, tag = "2")]
    pub timestamp: Option<UnsignedValue>,
    #[prost(message, optional, tag = "3")]
    pub transaction: Option<Transaction>,
}

///
/// Transaction
///
/// Transfer operation and required transaction metadata.
///

#[derive(Clone, prost::Message, PartialEq)]
pub(super) struct Transaction {
    #[prost(message, optional, tag = "3")]
    pub send: Option<Send>,
    #[prost(message, optional, tag = "4")]
    pub memo: Option<UnsignedValue>,
    #[prost(message, optional, tag = "6")]
    pub created_at_time: Option<UnsignedValue>,
}

///
/// Send
///
/// Transfer accounts, amount and required fee wrapper.
///

#[derive(Clone, prost::Message, PartialEq)]
pub(super) struct Send {
    #[prost(message, optional, tag = "1")]
    pub from: Option<BytesValue>,
    #[prost(message, optional, tag = "2")]
    pub to: Option<BytesValue>,
    #[prost(message, optional, tag = "3")]
    pub amount: Option<UnsignedValue>,
    #[prost(message, optional, tag = "4")]
    pub fee: Option<UnsignedValue>,
}

// The upstream Hash/AccountIdentifier and Tokens/Memo/TimeStamp wrappers each
// use field 1 with the wire type represented here. Required presence is checked
// by the owner; absent wrappers never become default fees or timestamps.
///
/// BytesValue
///
/// Wire-compatible hash or account-identifier bytes wrapper.
///

#[derive(Clone, prost::Message, PartialEq)]
pub(super) struct BytesValue {
    #[prost(bytes = "vec", tag = "1")]
    pub value: Vec<u8>,
}

///
/// UnsignedValue
///
/// Wire-compatible Tokens, Memo or TimeStamp scalar wrapper.
///

#[derive(Clone, prost::Message, PartialEq)]
pub(super) struct UnsignedValue {
    #[prost(uint64, tag = "1")]
    pub value: u64,
}
