use super::clap::{parse_matches, required_string, typed_option, value_arg};
use clap::Command;
use std::ffi::OsString;

#[test]
fn typed_helpers_read_values_from_one_clap_match_tree() {
    let command = Command::new("icq")
        .arg(value_arg("name").required(true))
        .arg(
            value_arg("count")
                .long("count")
                .value_parser(clap::value_parser!(u32)),
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

    assert_eq!(required_string(&matches, "name"), "report");
    assert_eq!(typed_option::<u32>(&matches, "count"), Some(3));
}
