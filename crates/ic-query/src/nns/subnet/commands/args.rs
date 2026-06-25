use crate::{
    cli::clap::{flag_arg, value_arg},
    subnet_catalog::{GeographicScope, SubnetKind, SubnetSpecialization},
};

#[cfg(test)]
pub(in crate::nns) const DEFAULT_RANGE_LIMIT: usize = 50;

pub(super) const DEFAULT_RANGE_LIMIT_ARG: &str = "50";
pub(super) const INFO_INPUT_VALUE_NAME: &str = "subnet|canister|subnet-prefix";
pub(super) const INFO_INPUT_HELP: &str = "Subnet/canister principal or unique subnet prefix";

pub(super) fn kind_arg() -> clap::Arg {
    value_arg("kind")
        .long("kind")
        .value_name("kind")
        .value_parser(clap::value_parser!(SubnetKind))
        .help("Filter by subnet kind: application, cloud_engine, system, or unknown")
}

pub(super) fn specialization_arg() -> clap::Arg {
    value_arg("specialization")
        .long("specialization")
        .value_name("specialization")
        .value_parser(clap::value_parser!(SubnetSpecialization))
        .help("Filter by specialization: none, fiduciary, european, or unknown")
}

pub(super) fn geo_arg() -> clap::Arg {
    value_arg("geo")
        .long("geo")
        .value_name("scope")
        .value_parser(clap::value_parser!(GeographicScope))
        .help("Filter by geographic scope: global, europe, or unknown")
}

pub(super) fn show_ranges_arg() -> clap::Arg {
    flag_arg("show-ranges")
        .long("show-ranges")
        .help("Show cached routing ranges after the subnet table")
}

pub(super) fn verbose_arg() -> clap::Arg {
    flag_arg("verbose")
        .long("verbose")
        .help("Show full subnet principals and catalog metadata in text output")
}

pub(super) fn range_limit_arg() -> clap::Arg {
    value_arg("range-limit")
        .long("range-limit")
        .value_name("n")
        .default_value(DEFAULT_RANGE_LIMIT_ARG)
        .value_parser(clap::builder::RangedU64ValueParser::<usize>::new().range(1u64..))
        .help("Maximum routing ranges to show per subnet in text output")
}

pub(super) fn range_offset_arg() -> clap::Arg {
    value_arg("range-offset")
        .long("range-offset")
        .value_name("n")
        .default_value("0")
        .value_parser(clap::value_parser!(usize))
        .help("Routing range offset for text output")
}
