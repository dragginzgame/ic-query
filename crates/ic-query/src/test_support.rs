use std::path::PathBuf;

pub fn temp_dir(prefix: &str) -> PathBuf {
    ic_host_fs::testing::create_fixture_directory(&std::env::temp_dir(), prefix)
        .expect("create private fixture directory")
}
