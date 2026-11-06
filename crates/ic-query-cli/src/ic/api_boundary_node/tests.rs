use super::*;
use crate::cli::{clap::render_help, common::OutputFormat};
use crate::ic::tests::parse_test_options;

#[test]
fn usage_discloses_certificate_authority_and_semantic_limits() {
    let family = render_help(command());
    let list = render_help(list_command());

    assert!(family.contains("Usage: api-boundary-node [COMMAND]"));
    assert!(family.contains("list"));
    assert!(list.contains("exactly one response-bounded IC read_state request"));
    assert!(list.contains("built-in mainnet root key"));
    assert!(list.contains("HTTP gateways"));
    assert!(list.contains("operational"));
    assert!(list.contains("health"));
}

#[test]
fn options_preserve_format_and_endpoint() {
    let options = parse_test_options(
        list_command(),
        &["--json", "--source-endpoint", "https://example.com"],
        SourceOutputOptions::from_matches,
    )
    .expect("API boundary-node options");

    assert_eq!(options.format(), OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://example.com");
}
