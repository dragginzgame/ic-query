const COMPACT_NEURON_ID_CHARS: usize = 8;

pub(super) fn neuron_id_for_list(value: &str, verbose: bool) -> String {
    if verbose || value == "-" {
        value.to_string()
    } else {
        value.chars().take(COMPACT_NEURON_ID_CHARS).collect()
    }
}
