use super::*;
use crate::cli::clap::{parse_matches, render_help};
use std::ffi::OsString;

#[test]
fn every_status_subject_advertises_the_shared_options() {
    for subject in [
        OperationalStatusSubject::Node,
        OperationalStatusSubject::NodeProvider,
        OperationalStatusSubject::Subnet,
    ] {
        let help = render_help(command(subject));
        assert!(help.contains("--all"));
        assert!(help.contains("--json"));
        assert!(help.contains("--refresh"));
        assert!(help.contains("--source-endpoint"));
        assert!(help.contains("Cache-backed read"));
    }
}

#[test]
fn status_options_parse_target_and_view_flags() {
    let matches = parse_matches(
        command(OperationalStatusSubject::Subnet),
        [
            OsString::from("tdb26"),
            OsString::from("--all"),
            OsString::from("--json"),
            OsString::from("--refresh"),
            OsString::from("--source-endpoint"),
            OsString::from("https://example.test/api/v3"),
        ],
    )
    .expect("parse status options");
    let (format, request) = status_request(&matches, "ic", PathBuf::from("/cache"), 1_780_531_200);

    assert_eq!(request.view.target.as_deref(), Some("tdb26"));
    assert_eq!(format, OutputFormat::Json);
    assert!(request.view.include_all);
    assert!(request.force_refresh);
    assert_eq!(
        request.refresh.source_endpoint,
        "https://example.test/api/v3"
    );
}
