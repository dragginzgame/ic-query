//! Bounded ICRC-3 value admission and contiguous certified block-chain validation.
//!
//! Callers authenticate the tip and interpret transaction semantics; no spending authority lives here.

pub mod icp;

#[cfg(test)]
mod tests;

use crate::certificate::CertifiedLedgerTip;
use candid::{CandidType, Nat};
use icrc_ledger_types::icrc::generic_value::Value;
use serde::Deserialize;
use thiserror::Error;

///
/// LedgerBlock
///
/// Passive standard generic-value block returned by a ledger's Candid query.
///

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct LedgerBlock {
    /// Ledger-assigned block index, before bounded admission.
    pub id: Nat,
    /// Uninterpreted generic block value returned by the ledger.
    pub block: Value,
}

///
/// BlockEvidenceLimits
///
/// Caller-selected total block, value-node, scalar/key-byte and nesting budgets.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockEvidenceLimits {
    /// Maximum number of blocks admitted in the complete supplied slice.
    pub blocks: usize,
    /// Maximum total generic-value nodes across all supplied blocks.
    pub nodes: usize,
    /// Maximum total scalar payload and map-key bytes across all blocks.
    pub bytes: usize,
    /// Maximum nested value levels, including the root of each block.
    pub depth: u8,
}

///
/// BlockEvidenceError
///
/// Typed evidence refusal with no conclusion about payment execution or accounting.
///

#[derive(Debug, Error, Eq, PartialEq)]
pub enum BlockEvidenceError {
    /// Admission exceeds the selected block, node, byte or nesting budget.
    #[error("block evidence exceeds its work budget")]
    BudgetExceeded,
    /// A block index is out of range or the requested certified chain does not match.
    #[error("block evidence does not match the certified contiguous chain")]
    ChainMismatch,
    /// A signed integer exceeds the range supported by the canonical ICRC hasher.
    #[error("block value is unsupported by the canonical ICRC hasher")]
    UnsupportedValue,
}

///
/// BoundedLedgerBlocks
///
/// Immutable values admitted before authentication/hashing; custody prevents later mutation.
///

#[derive(Debug)]
pub struct BoundedLedgerBlocks<'a> {
    blocks: &'a [LedgerBlock],
}

impl<'a> BoundedLedgerBlocks<'a> {
    /// Admit the complete value set without hashing or interpreting transactions.
    ///
    /// # Errors
    /// Refuses work budgets, out-of-range indexes and values that could panic the hasher.
    pub fn admit(
        blocks: &'a [LedgerBlock],
        limits: BlockEvidenceLimits,
    ) -> Result<Self, BlockEvidenceError> {
        if blocks.len() > limits.blocks {
            return Err(BlockEvidenceError::BudgetExceeded);
        }
        let mut nodes = limits.nodes;
        let mut bytes = limits.bytes;
        for block in blocks {
            if block.id.0.bits() > 128 {
                return Err(BlockEvidenceError::ChainMismatch);
            }
            check_value(&block.block, limits.depth, &mut nodes, &mut bytes)?;
        }
        Ok(Self { blocks })
    }

    /// Prove the exact descending chain from an independently authenticated tip to an index.
    ///
    /// # Errors
    /// Refuses gaps, duplicates, changed hashes/parent links or an incorrect target locator.
    pub fn verify_chain(
        &self,
        tip: &CertifiedLedgerTip,
        target: u128,
    ) -> Result<&'a Value, BlockEvidenceError> {
        let expected_len = tip.index.checked_sub(target).and_then(|n| n.checked_add(1));
        if self.blocks.is_empty() || expected_len != Some(self.blocks.len() as u128) {
            return Err(BlockEvidenceError::ChainMismatch);
        }
        let mut expected_hash = tip.hash;
        for (offset, block) in self.blocks.iter().enumerate() {
            let expected_index = tip.index - offset as u128;
            if block.id != expected_index || block.block.hash() != expected_hash {
                return Err(BlockEvidenceError::ChainMismatch);
            }
            let Value::Map(fields) = &block.block else {
                return Err(BlockEvidenceError::ChainMismatch);
            };
            if expected_index == 0 {
                if fields.contains_key("phash") {
                    return Err(BlockEvidenceError::ChainMismatch);
                }
            } else {
                let Some(Value::Blob(bytes)) = fields.get("phash") else {
                    return Err(BlockEvidenceError::ChainMismatch);
                };
                expected_hash = bytes
                    .as_ref()
                    .try_into()
                    .map_err(|_| BlockEvidenceError::ChainMismatch)?;
            }
        }
        self.blocks
            .last()
            .map(|block| &block.block)
            .ok_or(BlockEvidenceError::ChainMismatch)
    }
}

fn check_value(
    value: &Value,
    depth: u8,
    nodes: &mut usize,
    bytes: &mut usize,
) -> Result<(), BlockEvidenceError> {
    *nodes = nodes
        .checked_sub(1)
        .ok_or(BlockEvidenceError::BudgetExceeded)?;
    let depth = depth
        .checked_sub(1)
        .ok_or(BlockEvidenceError::BudgetExceeded)?;
    let scalar_bytes = match value {
        Value::Blob(value) => value.len(),
        Value::Text(value) => value.len(),
        Value::Nat64(_) => size_of::<u64>(),
        Value::Nat(value) => usize::try_from(value.0.bits().div_ceil(8))
            .map_err(|_| BlockEvidenceError::BudgetExceeded)?,
        Value::Int(value) => {
            i128::try_from(&value.0).map_err(|_| BlockEvidenceError::UnsupportedValue)?;
            usize::try_from(value.0.bits().div_ceil(8))
                .map_err(|_| BlockEvidenceError::BudgetExceeded)?
        }
        Value::Array(values) => {
            for value in values {
                check_value(value, depth, nodes, bytes)?;
            }
            0
        }
        Value::Map(values) => {
            for (key, value) in values {
                *bytes = bytes
                    .checked_sub(key.len())
                    .ok_or(BlockEvidenceError::BudgetExceeded)?;
                check_value(value, depth, nodes, bytes)?;
            }
            0
        }
    };
    *bytes = bytes
        .checked_sub(scalar_bytes)
        .ok_or(BlockEvidenceError::BudgetExceeded)?;
    Ok(())
}
