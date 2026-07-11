use crate::nns::leaf::NnsLeafCommandSpec;

const DATA_CENTER_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns data-center list
  icq nns data-center list --verbose
  icq --network ic nns data-center list --format json

Force-refresh cached native NNS data:
  icq nns data-center refresh";
const DATA_CENTER_INFO_HELP_AFTER: &str = "\
Examples:
  icq nns data-center info <data-center>
  icq nns data-center info <data-center-prefix>
  icq --network ic nns data-center info <data-center> --format json

Force-refresh cached native NNS data:
  icq nns data-center refresh";
const DATA_CENTER_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns data-center refresh
  icq --network ic nns data-center refresh --format json
  icq nns data-center refresh --dry-run --output .icq/data-center/ic/data-centers.preview.json";

pub(super) const DATA_CENTER_SPEC: NnsLeafCommandSpec = NnsLeafCommandSpec {
    command_name: "data-center",
    bin_name: "icq nns data-center",
    about: "Inspect NNS data-center metadata",
    list_about: "List cached mainnet NNS data centers",
    info_about: "Show one cached mainnet NNS data center",
    refresh_about: "Force-refresh and cache NNS data-center metadata",
    list_help_after: DATA_CENTER_LIST_HELP_AFTER,
    info_help_after: DATA_CENTER_INFO_HELP_AFTER,
    refresh_help_after: DATA_CENTER_REFRESH_HELP_AFTER,
    input_value_name: "data-center|data-center-prefix",
    input_help: "Data-center id or unique data-center id prefix",
    list_source_help: "IC API endpoint used if the data-center cache is missing",
    info_source_help: "IC API endpoint used if the data-center cache is missing",
    refresh_source_help: "IC API endpoint used for native NNS registry queries",
    verbose_help: "Show GPS coordinates and registry metadata in text output",
    dry_run_help: "Fetch and validate without replacing the cached data-center report",
    output_help: "Also write the fetched data-center JSON to this path",
};
