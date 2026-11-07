use ic_host_fs::durable::{PublicationMode, WriteOptions, prepare_private_parents, write_bytes};

pub fn scratch_root() -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .join("target/tmp");
    std::fs::create_dir_all(&root).expect("create workspace integration scratch");
    root
}

#[allow(
    dead_code,
    reason = "Separate integration-test binaries use different subsets of the shared fixtures."
)]
pub fn temp_root(name: &str) -> std::path::PathBuf {
    ic_host_fs::testing::create_fixture_directory(&scratch_root(), name)
        .expect("create private physical fixture directory")
}

#[allow(
    dead_code,
    reason = "Separate integration-test binaries use different subsets of the shared fixtures."
)]
pub fn write_json_cache(
    root: &std::path::Path,
    path: &std::path::Path,
    value: &impl serde::Serialize,
) {
    assert!(
        path.starts_with(root),
        "cache fixture stays beneath its root"
    );
    prepare_private_parents(path.parent().expect("cache parent"))
        .expect("create private cache parents");
    let bytes = serde_json::to_vec_pretty(value).expect("serialize fixture cache");
    write_bytes(
        path,
        WriteOptions {
            mode: PublicationMode::Replace,
            permissions: 0o600,
        },
        &bytes,
    )
    .expect("write private fixture cache");
}
