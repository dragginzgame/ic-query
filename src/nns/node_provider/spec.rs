use crate::nns::leaf::NnsLeafCommandSpec;

const NODE_PROVIDER_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns node-provider list
  icq nns node-provider list --verbose
  icq --network ic nns node-provider list --format json

Force-refresh cached native NNS data:
  icq nns node-provider refresh";
const NODE_PROVIDER_INFO_HELP_AFTER: &str = "\
Examples:
  icq nns node-provider info <node-provider>
  icq nns node-provider info <node-provider-prefix>
  icq --network ic nns node-provider info <node-provider> --format json

Force-refresh cached native NNS data:
  icq nns node-provider refresh";
const NODE_PROVIDER_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns node-provider refresh
  icq --network ic nns node-provider refresh --format json
  icq nns node-provider refresh --dry-run --output .icq/node-provider/ic/providers.preview.json";

pub(super) const NODE_PROVIDER_SPEC: NnsLeafCommandSpec = NnsLeafCommandSpec {
    command_name: "node-provider",
    bin_name: "icq nns node-provider",
    about: "Inspect NNS node-provider metadata",
    list_about: "List cached mainnet NNS node providers",
    info_about: "Show one cached mainnet NNS node provider",
    refresh_about: "Force-refresh and cache NNS node-provider metadata",
    list_help_after: NODE_PROVIDER_LIST_HELP_AFTER,
    info_help_after: NODE_PROVIDER_INFO_HELP_AFTER,
    refresh_help_after: NODE_PROVIDER_REFRESH_HELP_AFTER,
    input_value_name: "node-provider|node-provider-prefix",
    input_help: "Node-provider principal or unique node-provider principal prefix",
    list_source_help: "IC API endpoint used if the node-provider cache is missing",
    info_source_help: "IC API endpoint used if the node-provider cache is missing",
    refresh_source_help: "IC API endpoint used for native NNS governance and registry queries",
    verbose_help: "Show full node-provider principals and reward-account metadata in text output",
    dry_run_help: "Fetch and validate without replacing the cached node-provider report",
    output_help: "Also write the fetched node-provider JSON to this path",
};
