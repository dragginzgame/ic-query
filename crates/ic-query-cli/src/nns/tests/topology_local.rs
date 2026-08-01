use super::*;

#[test]
fn topology_commands_reject_non_mainnet_network() {
    for command in [
        "summary",
        "coverage",
        "versions",
        "health",
        "gaps",
        "capacity",
        "regions",
        "providers",
        "refresh",
    ] {
        let err = run([
            OsString::from("topology"),
            OsString::from(command),
            OsString::from("--__icq-network"),
            OsString::from("local"),
        ])
        .expect_err("local rejected");

        let message = err.to_string();
        assert!(message.contains("supports only the mainnet `ic` network"));
        assert!(message.contains(&format!("icq --network ic nns topology {command}")));
    }
}
