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
mod tests {
    use super::*;
    use ic_platform::candid::{contract::Error, nesting::NestingError};

    #[derive(Debug, PartialEq, ic_platform::candid::contract::CandidType, serde::Deserialize)]
    #[candid_path("ic_platform::candid::contract")]
    enum Tree {
        Leaf(u64),
        Branch(Vec<Self>),
    }

    #[test]
    fn external_nat64_wire_vector_is_unchanged() {
        let bytes = b"DIDL\x00\x01\x78\x2a\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(
            ic_platform::candid::contract::encode_one(42_u64).unwrap(),
            bytes
        );
        assert_eq!(decode_reply::<u64>(bytes).unwrap(), 42);
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
        let bytes = ic_platform::candid::contract::encode_one(&value).unwrap();
        assert_eq!(decode_reply::<Tree>(&bytes).unwrap(), value);
    }

    #[test]
    fn rejects_amplified_skipping_in_a_small_reply() {
        let ordinary = ic_platform::candid::contract::encode_args((42u64, "unused")).unwrap();
        assert_eq!(decode_reply::<u64>(&ordinary).unwrap(), 42);
        let bytes = ic_platform::candid::contract::encode_args((42u64, vec![(); 100_000])).unwrap();
        assert!(bytes.len() < 32);
        assert!(decode_reply::<u64>(&bytes).is_err());
    }

    #[test]
    fn skipped_external_values_retain_platform_nesting_admission_and_typed_errors() {
        let shallow = Tree::Branch(vec![Tree::Leaf(1)]);
        let deep = (0..16).fold(Tree::Leaf(1), |child, _| Tree::Branch(vec![child]));
        let ordinary = ic_platform::candid::contract::encode_args((42_u64, shallow)).unwrap();
        assert_eq!(decode_reply::<u64>(&ordinary).unwrap(), 42);

        let nested = ic_platform::candid::contract::encode_args((42_u64, deep)).unwrap();
        let Error::Custom(error) = decode_reply::<u64>(&nested).unwrap_err() else {
            panic!("expected Platform's typed nesting refusal");
        };
        assert_eq!(
            error.downcast_ref::<NestingError>(),
            Some(&NestingError::ValueDepth { maximum: 16 }),
        );
    }

    #[test]
    fn rejects_amplified_decoding_in_a_small_reply() {
        let bytes = ic_platform::candid::contract::encode_one(vec![(); 1_000_000]).unwrap();
        assert!(bytes.len() < 32);
        assert!(decode_reply::<Vec<()>>(&bytes).is_err());
    }

    #[test]
    fn accepts_a_large_byte_vector() {
        let value = vec![7u8; 8 * 1024 * 1024 - 32];
        let bytes = ic_platform::candid::contract::encode_one(&value).unwrap();
        assert_eq!(decode_reply::<Vec<u8>>(&bytes).unwrap(), value);
    }

    #[test]
    fn rejects_an_excessive_type_table_before_reading_entries() {
        // DIDL followed by the unsigned LEB128 type count 4097.
        let bytes = b"DIDL\x81\x20";
        let Error::Custom(error) = decode_reply::<u64>(bytes).unwrap_err() else {
            panic!("expected Platform's typed type-table refusal");
        };
        assert_eq!(
            error.downcast_ref::<NestingError>(),
            Some(&NestingError::Types {
                actual: 4_097,
                maximum: MAX_TYPE_ENTRIES,
            }),
        );
    }
}
