//! Bounded Candid reply decoding shared by native and canister adapters.

use ic_platform::candid::contract::CandidType;
use ic_platform::candid::{DecodeError, DecodeLimits, decode_one};
use serde::Deserialize;

// Charge proportional wire work with headroom for wide records and callbacks;
// tiny messages cannot amplify decoding or skipped values without a finite bound.
const BASE_DECODING_QUOTA: usize = 1_000_000;
const MAX_DECODING_QUOTA: usize = 256_000_000;
const SKIPPING_QUOTA: usize = 100_000;
const MAX_TYPE_ENTRIES: usize = 4_096;
const MAX_HEADER_BYTES: usize = 64 * 1_024;

pub fn decode_reply<'a, T: Deserialize<'a> + CandidType>(
    bytes: &'a [u8],
) -> Result<T, ic_platform::candid::contract::Error> {
    let quota = bytes
        .len()
        .saturating_mul(32)
        .saturating_add(BASE_DECODING_QUOTA)
        .min(MAX_DECODING_QUOTA);
    decode_one(
        bytes,
        DecodeLimits {
            // Transport owners admit wire size before calling this shared policy.
            reply_bytes: bytes.len(),
            decoding_quota: quota,
            skipping_quota: SKIPPING_QUOTA,
            type_entries: MAX_TYPE_ENTRIES,
            header_bytes: MAX_HEADER_BYTES,
        },
    )
    .map_err(|error| match error {
        DecodeError::Candid(source) => source,
        error @ DecodeError::ReplyTooLarge { .. } => {
            ic_platform::candid::contract::Error::msg(error.to_string())
        }
    })
}

#[cfg(test)]
mod tests;
