use super::*;
use crate::cli::{
    clap::{parse_matches, render_help},
    common::{OutputFormat, SOURCE_ENDPOINT_ARG, output_format},
};
use std::ffi::OsString;

#[test]
fn help_discloses_type4_scope_authority_and_bounds() {
    let namespace = render_help(command());
    assert!(namespace.contains("Usage: node [COMMAND]"));
    assert!(namespace.contains("info"));
    assert!(namespace.contains("list"));

    let info = render_help(info_command());
    assert!(info.contains("<node-id>"));
    assert!(info.contains("requires the"));
    assert!(info.contains("returned reward type to be Type4"));
    assert!(info.contains("uncertified"));

    let list = render_help(list_command());
    assert!(list.contains("explicit Type4"));
    assert!(list.contains("all four currently documented statuses"));
    assert!(list.contains("10,000"));
    assert!(list.contains("--node-provider-id <principal>"));
}

#[test]
fn list_options_preserve_provider_endpoint_and_format() {
    let matches = parse_matches(
        list_command(),
        [
            "--node-provider-id",
            "bvcsg-3od6r-jnydw-eysln-aql7w-td5zn-ay5m6-sibd2-jzojt-anwag-mqe",
            "--json",
        ]
        .into_iter()
        .map(OsString::from),
    )
    .expect("parse Type4 node list options");

    assert_eq!(
        required_string(&matches, SOURCE_ENDPOINT_ARG),
        DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT
    );
    assert!(string_option(&matches, "node-provider-id").is_some());
    assert_eq!(output_format(&matches), OutputFormat::Json);
}
