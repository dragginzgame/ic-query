use super::*;
use crate::cli::clap::{parse_matches, render_help};
use crate::cli::common::SOURCE_ENDPOINT_ARG;
use std::ffi::OsString;

#[test]
fn usage_describes_bounded_native_reports() {
    let text = render_help(command());
    assert!(text.contains("Usage: system [COMMAND]"));
    assert!(text.contains("xdr"));
    assert!(text.contains("cycles"));

    for text in [
        render_help(report_command(
            "xdr",
            "Show the certified CMC ICP/XDR conversion rate",
            XDR_HELP_AFTER,
        )),
        render_help(report_command(
            "cycles",
            "Show cycles conversions derived from the certified CMC rate",
            CYCLES_HELP_AFTER,
        )),
    ] {
        assert!(text.contains("--source-endpoint <url>"));
        assert!(text.contains("--json"));
        assert!(text.contains(COLLECTION_MODE_LIVE));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn help_and_version_do_not_make_live_calls() {
    for args in [
        &["system", "--help"][..],
        &["system", "xdr", "--help"],
        &["system", "cycles", "--help"],
        &["system", "--version"],
    ] {
        assert!(crate::run(args.iter().map(OsString::from)).await.is_ok());
    }
}

#[test]
fn report_options_default_to_mainnet_and_native_endpoint() {
    let matches = parse_matches(
        report_command("xdr", "test", XDR_HELP_AFTER),
        Vec::<OsString>::new(),
    )
    .expect("parse default CMC options");

    assert_eq!(
        matches
            .get_one::<String>(SOURCE_ENDPOINT_ARG)
            .expect("validated source endpoint"),
        DEFAULT_CMC_SOURCE_ENDPOINT
    );
    assert_eq!(
        output_format(&matches),
        crate::cli::common::OutputFormat::Text
    );
}
