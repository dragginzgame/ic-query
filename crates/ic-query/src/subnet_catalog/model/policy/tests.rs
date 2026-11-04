#[test]
fn policy_table_preserves_existing_fingerprint() {
    assert_eq!(
        super::classification_policy_digest(),
        "a3a544eb8c5a54a93b06583a16e3952598cee5ec7d9629e2678c5ed85bd58a2c"
    );
}
