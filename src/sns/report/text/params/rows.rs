pub(in crate::sns::report::text::params) fn parameter_row(
    name: &str,
    value: String,
) -> [String; 2] {
    [name.to_string(), value]
}
