use super::*;

#[test]
fn presentation_flags_are_independent_of_report_requests_and_json_selection() {
    let command = clap::Command::new("fixture")
        .arg(flag_arg("verbose").long("verbose"))
        .arg(flag_arg("json").long("json"));
    let defaults = command.clone().try_get_matches_from(["fixture"]).unwrap();
    assert_eq!(
        report_text_options(&defaults),
        ic_query_text::TextOptions::default()
    );
    let matches = command
        .try_get_matches_from(["fixture", "--verbose", "--json"])
        .unwrap();
    assert_eq!(output_format(&matches), OutputFormat::Json);
    assert_eq!(
        report_text_options(&matches),
        ic_query_text::TextOptions { verbose: true }
    );
}
