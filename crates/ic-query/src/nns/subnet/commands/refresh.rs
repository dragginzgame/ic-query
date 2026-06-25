use crate::{
    cli::clap::flag_arg, nns::leaf, subnet_catalog::DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
};
use clap::Command as ClapCommand;

const REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns subnet refresh
  icq --network ic nns subnet refresh --format json
  icq nns subnet refresh --dry-run --output .icq/subnet-catalog/ic/catalog.preview.json";

pub(in crate::nns::subnet) fn refresh_command() -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name("icq nns subnet refresh")
        .about("Force-refresh and cache NNS subnet metadata")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the NNS registry query"),
        )
        .arg(leaf::refresh_lock_stale_after_arg())
        .arg(
            flag_arg("dry-run")
                .long("dry-run")
                .help("Fetch and validate without replacing the cached catalog"),
        )
        .arg(leaf::output_path_arg().help("Also write the fetched catalog JSON to this path"))
        .arg(leaf::network_arg())
        .after_help(REFRESH_HELP_AFTER)
}
