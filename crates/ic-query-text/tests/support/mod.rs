pub fn scratch_root() -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .join("target/tmp");
    std::fs::create_dir_all(&root).expect("create workspace integration scratch");
    root
}
