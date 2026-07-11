pub fn assert_snapshot(name: &str, actual: &str, expected: &str) {
    assert_eq!(actual, expected, "{name} snapshot changed");
}
