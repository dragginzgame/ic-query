pub fn compact_principal(value: &str) -> String {
    value.chars().take(5).collect()
}
