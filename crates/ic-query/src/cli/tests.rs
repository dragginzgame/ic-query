use super::clap::{parse_required_subcommand, passthrough_subcommand};
use clap::{Command, error::ErrorKind};
use std::ffi::OsString;

#[test]
fn parse_required_subcommand_reports_missing_subcommand() {
    let error = parse_required_subcommand(Command::new("icq"), []).expect_err("missing command");

    assert_eq!(error.kind(), ErrorKind::MissingSubcommand);
}

#[test]
fn parse_required_subcommand_returns_passthrough_args() {
    let command = Command::new("icq").subcommand(passthrough_subcommand(Command::new("sns")));

    let (name, args) = parse_required_subcommand(
        command,
        [
            OsString::from("sns"),
            OsString::from("neurons"),
            OsString::from("--limit"),
            OsString::from("50"),
        ],
    )
    .expect("parse command");

    assert_eq!(name, "sns");
    assert_eq!(
        args,
        vec![
            OsString::from("neurons"),
            OsString::from("--limit"),
            OsString::from("50"),
        ],
    );
}
