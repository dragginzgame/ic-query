use super::*;
use crate::cli::clap::render_help;
use std::ffi::OsString;

#[test]
fn help_describes_local_only_cache_status() {
    let help = render_help(command());
    assert!(help.contains("status"));
    let status = command().find_subcommand("status").expect("status").clone();
    let help = render_help(status);
    assert!(help.contains("--json"));
    assert!(help.contains("invalid recovery"));
    assert!(help.contains(COLLECTION_MODE_CACHE_ONLY));
}

#[tokio::test(flavor = "current_thread")]
async fn help_does_not_inspect_the_filesystem() {
    for args in [&["cache", "--help"][..], &["cache", "status", "--help"]] {
        assert!(crate::run(args.iter().map(OsString::from)).await.is_ok());
    }
}
