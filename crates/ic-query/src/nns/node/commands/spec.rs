use crate::nns::leaf::NnsLeafCommandSpec;

const NODE_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns node list
  icq nns node list --verbose
  icq --network ic nns node list --format json
  icq nns node list --data-center zh2
  icq nns node list --node-provider 7at4h
  icq nns node list --subnet tdb26 --kind system

Force-refresh cached native NNS data:
  icq nns node refresh";
const NODE_INFO_HELP_AFTER: &str = "\
Examples:
  icq nns node info <node>
  icq nns node info <node-prefix>
  icq --network ic nns node info <node> --format json

Force-refresh cached native NNS data:
  icq nns node refresh";
const NODE_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns node refresh
  icq --network ic nns node refresh --format json
  icq nns node refresh --dry-run --output .icq/node/ic/nodes.preview.json";

pub(in crate::nns::node) const NODE_SPEC: NnsLeafCommandSpec = NnsLeafCommandSpec {
    command_name: "node",
    bin_name: "icq nns node",
    about: "Inspect NNS node metadata",
    list_about: "List cached mainnet NNS nodes",
    info_about: "Show one cached mainnet NNS node",
    refresh_about: "Force-refresh and cache NNS node metadata",
    list_help_after: NODE_LIST_HELP_AFTER,
    info_help_after: NODE_INFO_HELP_AFTER,
    refresh_help_after: NODE_REFRESH_HELP_AFTER,
    input_value_name: "node|node-prefix",
    input_help: "Node principal or unique node principal prefix",
    list_source_help: "IC API endpoint used if the node cache is missing",
    info_source_help: "IC API endpoint used if the node cache is missing",
    refresh_source_help: "IC API endpoint used for native NNS registry queries",
    verbose_help: "Show full node principals and registry metadata in text output",
    dry_run_help: "Fetch and validate without replacing the cached node report",
    output_help: "Also write the fetched node JSON to this path",
};
