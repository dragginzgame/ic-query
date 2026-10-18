use crate::cli::clap::value_arg;
use std::path::PathBuf;

pub(in crate::nns) const INPUT_ARG: &str = "input";
pub(in crate::nns) const DRY_RUN_ARG: &str = "dry-run";
pub(in crate::nns) const OUTPUT_ARG: &str = "output";
pub(in crate::nns) const VERBOSE_ARG: &str = "verbose";

pub(in crate::nns) fn output_path_arg() -> clap::Arg {
    value_arg(OUTPUT_ARG)
        .long(OUTPUT_ARG)
        .value_name("path")
        .value_parser(clap::value_parser!(PathBuf))
}
