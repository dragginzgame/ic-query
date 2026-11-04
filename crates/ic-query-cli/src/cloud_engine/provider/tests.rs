use super::*;
use crate::cli::{
    clap::{parse_matches, render_help},
    common::{OutputFormat, SOURCE_ENDPOINT_ARG, output_format},
};
use std::ffi::OsString;

#[test]
fn help_discloses_provider_identity_authority_and_bounds() {
    let namespace = render_help(command());
    assert!(namespace.contains("Usage: icq cloud-engine provider [COMMAND]"));
    assert!(namespace.contains("info"));
    assert!(namespace.contains("list"));

    let info = render_help(info_command());
    assert!(info.contains("<node-provider-id>"));
    assert!(info.contains("not an engine canister principal"));
    assert!(info.contains("one exact official Dashboard request"));
    assert!(info.contains("no CloudEngine evidence"));

    let list = render_help(list_command());
    assert!(list.contains("complete node-provider"));
    assert!(list.contains("1,000"));
    assert!(list.contains("uncertified"));
    assert!(list.contains("--source-endpoint <url>"));
}

#[test]
fn report_options_default_to_dashboard_and_text() {
    for command in [
        list_command(),
        info_command().mut_arg("node-provider-id", |arg| arg.required(false)),
    ] {
        let matches =
            parse_matches(command, Vec::<OsString>::new()).expect("parse default provider options");
        assert_eq!(
            required_string(&matches, SOURCE_ENDPOINT_ARG),
            DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT
        );
        assert_eq!(output_format(&matches), OutputFormat::Text);
    }
}
