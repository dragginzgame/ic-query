use super::clap::{parse_matches, parsed_options, with_options};
use clap::{Args, Command};
use std::ffi::OsString;

pub fn render_help(mut command: Command) -> String {
    command.render_help().to_string()
}

pub fn parse_test_args(command: Command, args: &[&str]) -> Result<clap::ArgMatches, clap::Error> {
    parse_matches(command, args.iter().copied().map(OsString::from))
}

#[test]
fn typed_helpers_read_values_from_one_clap_match_tree() {
    /// Rust-only option documentation.
    ///
    /// The command owns its user-facing help.
    #[derive(Args)]
    struct Options {
        name: String,
        #[arg(long)]
        count: Option<u32>,
    }

    let command = with_options::<Options>(Command::new("icq").about("Inspect reports"));
    assert_eq!(
        command
            .get_about()
            .expect("command description")
            .to_string(),
        "Inspect reports"
    );
    assert!(command.get_long_about().is_none());
    let detailed = with_options::<Options>(Command::new("icq").long_about("Detailed report help"));
    assert!(detailed.get_about().is_none());
    assert_eq!(
        detailed
            .get_long_about()
            .expect("long description")
            .to_string(),
        "Detailed report help"
    );
    let matches = parse_matches(
        command,
        [
            OsString::from("report"),
            OsString::from("--count"),
            OsString::from("3"),
        ],
    )
    .expect("parse typed arguments");

    let options: Options = parsed_options(&matches);
    assert_eq!(options.name, "report");
    assert_eq!(options.count, Some(3));
}
