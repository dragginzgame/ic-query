use super::super::common::neuron_id_text;

pub(super) fn neuron_id_for_list(value: &str, verbose: bool) -> String {
    neuron_id_text(value, verbose)
}
