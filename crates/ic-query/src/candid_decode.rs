//! Bounded Candid reply decoding shared by native and canister adapters.

use candid::{CandidType, de::DecoderConfig};
use serde::Deserialize;

// Charge proportional wire work with headroom for wide records and callbacks;
// tiny messages cannot amplify decoding or skipped values without a finite bound.
const BASE_DECODING_QUOTA: usize = 1_000_000;
const MAX_DECODING_QUOTA: usize = 256_000_000;
const SKIPPING_QUOTA: usize = 100_000;
const MAX_TYPE_ENTRIES: usize = 4_096;
const MAX_HEADER_BYTES: usize = 64 * 1_024;

pub fn decode_reply<'a, T: Deserialize<'a> + CandidType>(bytes: &'a [u8]) -> candid::Result<T> {
    let quota = bytes
        .len()
        .saturating_mul(32)
        .saturating_add(BASE_DECODING_QUOTA)
        .min(MAX_DECODING_QUOTA);
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(quota)
        .set_skipping_quota(SKIPPING_QUOTA)
        .set_max_type_len(MAX_TYPE_ENTRIES)
        .set_max_header_len(MAX_HEADER_BYTES);
    candid::decode_one_with_config(bytes, &config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, candid::CandidType, serde::Deserialize)]
    enum Tree {
        Leaf(u64),
        Branch(Vec<Self>),
    }

    #[test]
    fn accepts_wide_recursive_typed_values() {
        let value = Tree::Branch(
            (0..10_000)
                .map(|index| {
                    Tree::Branch(vec![
                        Tree::Leaf(index),
                        Tree::Branch(vec![Tree::Leaf(index + 1)]),
                    ])
                })
                .collect(),
        );
        let bytes = candid::encode_one(&value).unwrap();
        assert_eq!(decode_reply::<Tree>(&bytes).unwrap(), value);
    }

    #[test]
    fn rejects_amplified_skipping_in_a_small_reply() {
        let bytes = candid::encode_args((42u64, vec![(); 100_000])).unwrap();
        assert!(bytes.len() < 32);
        assert!(decode_reply::<u64>(&bytes).is_err());
    }

    #[test]
    fn rejects_amplified_decoding_in_a_small_reply() {
        let bytes = candid::encode_one(vec![(); 1_000_000]).unwrap();
        assert!(bytes.len() < 32);
        assert!(decode_reply::<Vec<()>>(&bytes).is_err());
    }

    #[test]
    fn accepts_a_large_byte_vector() {
        let value = vec![7u8; 8 * 1024 * 1024 - 32];
        let bytes = candid::encode_one(&value).unwrap();
        assert_eq!(decode_reply::<Vec<u8>>(&bytes).unwrap(), value);
    }

    #[test]
    fn rejects_an_excessive_type_table_before_reading_entries() {
        // DIDL followed by the unsigned LEB128 type count 4097.
        let bytes = b"DIDL\x81\x20";
        let error = decode_reply::<u64>(bytes).unwrap_err();
        assert!(format!("{error:#}").contains("type table"));
    }
}
