use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()))
}

pub fn assert_snapshot(name: &str, actual: &str, expected: &str) {
    assert_eq!(actual, expected, "{name} snapshot changed");
}
