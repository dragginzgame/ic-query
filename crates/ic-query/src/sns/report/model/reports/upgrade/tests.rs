use super::SnsVersion;

#[test]
fn version_hash_admission_preserves_upgrade_policy() {
    for (hash, upgrade_valid) in [
        ("0a".repeat(32), true),
        ("0a".to_string(), true),
        ("0a".repeat(33), true),
        (String::new(), false),
        ("a".to_string(), false),
        ("AB".repeat(32), false),
        ("zz".repeat(32), false),
    ] {
        let version = SnsVersion {
            archive_wasm_hash_hex: hash.clone(),
            root_wasm_hash_hex: hash.clone(),
            swap_wasm_hash_hex: hash.clone(),
            ledger_wasm_hash_hex: hash.clone(),
            governance_wasm_hash_hex: hash.clone(),
            index_wasm_hash_hex: hash,
        };
        assert_eq!(version.validate_hashes("version").is_ok(), upgrade_valid);
    }
}
