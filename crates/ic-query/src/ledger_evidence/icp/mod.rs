//! Bounded decoding of current ICP Ledger protobuf transfer blocks.
//!
//! Decoding does not authenticate a block or authorize a payment. Callers retain
//! certificate verification, exact block selection and expected-transfer checks.

mod protobuf;
#[cfg(test)]
mod tests;

use prost::Message;
use thiserror::Error;

/// Transfer fields decoded from one canonical ICP block; not certified evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcpTransferFields {
    /// Source account identifier, including its checksum bytes.
    pub from: [u8; 32],
    /// Destination account identifier, including its checksum bytes.
    pub to: [u8; 32],
    /// Transferred amount in e8s.
    pub amount_e8s: u64,
    /// Charged transfer fee in e8s.
    pub fee_e8s: u64,
    /// Ledger memo; interpretation belongs to the caller.
    pub memo: u64,
    /// Transaction creation time in nanoseconds.
    pub created_at_time_ns: u64,
}

/// Structural refusal before a caller interprets decoded transfer fields.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum IcpBlockError {
    /// The supplied bytes exceed the caller's allocation budget.
    #[error("ICP block exceeds its byte budget")]
    BudgetExceeded,
    /// A present source or destination account has the wrong byte length.
    #[error("ICP transfer account identifier is not 32 bytes")]
    InvalidAccount,
    /// The block is malformed, noncanonical, incomplete or outside the supported send schema.
    #[error("ICP block has an unsupported or noncanonical protobuf schema")]
    UnsupportedBlock,
}

/// Decode one transfer after checking its byte budget, exact protobuf encoding and parent shape.
///
/// `index` is the independently selected ledger block index. Index zero requires
/// no parent; later indices require a 32-byte parent hash. This does not prove
/// that the supplied block occupies that index or connects to a certified tip.
/// Unknown fields, non-send operations, missing wrappers and duplicate singular
/// fields reject. No missing fee, memo or timestamp receives a default value.
pub fn decode_transfer(
    bytes: &[u8],
    index: u64,
    maximum_bytes: usize,
) -> Result<IcpTransferFields, IcpBlockError> {
    if bytes.len() > maximum_bytes {
        return Err(IcpBlockError::BudgetExceeded);
    }
    let block = protobuf::Block::decode(bytes).map_err(|_| IcpBlockError::UnsupportedBlock)?;
    if block.encode_to_vec() != bytes || block.timestamp.is_none() {
        return Err(IcpBlockError::UnsupportedBlock);
    }
    match (&block.parent_hash, index) {
        (None, 0) => {}
        (Some(hash), 1..) if hash.value.len() == 32 => {}
        _ => return Err(IcpBlockError::UnsupportedBlock),
    }
    let transaction = block.transaction.ok_or(IcpBlockError::UnsupportedBlock)?;
    let send = transaction.send.ok_or(IcpBlockError::UnsupportedBlock)?;
    Ok(IcpTransferFields {
        from: send
            .from
            .ok_or(IcpBlockError::UnsupportedBlock)?
            .value
            .try_into()
            .map_err(|_| IcpBlockError::InvalidAccount)?,
        to: send
            .to
            .ok_or(IcpBlockError::UnsupportedBlock)?
            .value
            .try_into()
            .map_err(|_| IcpBlockError::InvalidAccount)?,
        amount_e8s: send.amount.ok_or(IcpBlockError::UnsupportedBlock)?.value,
        fee_e8s: send.fee.ok_or(IcpBlockError::UnsupportedBlock)?.value,
        memo: transaction
            .memo
            .ok_or(IcpBlockError::UnsupportedBlock)?
            .value,
        created_at_time_ns: transaction
            .created_at_time
            .ok_or(IcpBlockError::UnsupportedBlock)?
            .value,
    })
}
