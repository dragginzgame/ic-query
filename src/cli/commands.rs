#[derive(Clone, Copy, Debug)]
pub struct CommandFamily {
    pub name: &'static str,
    pub about: &'static str,
    pub accepts_global_network: fn(&[std::ffi::OsString]) -> bool,
}

pub const COMMAND_FAMILIES: &[CommandFamily] = &[
    CommandFamily {
        name: "nns",
        about: "Inspect NNS metadata",
        accepts_global_network: nns_accepts_global_network,
    },
    CommandFamily {
        name: "sns",
        about: "Inspect SNS metadata",
        accepts_global_network: sns_accepts_global_network,
    },
];

pub fn command_family(name: &str) -> Option<&'static CommandFamily> {
    COMMAND_FAMILIES.iter().find(|family| family.name == name)
}

fn nns_accepts_global_network(tail: &[std::ffi::OsString]) -> bool {
    matches!(
        tail.first().and_then(|arg| arg.to_str()),
        Some(
            "data-center"
                | "node"
                | "node-operator"
                | "node-provider"
                | "registry"
                | "subnet"
                | "topology"
        )
    )
}

fn sns_accepts_global_network(tail: &[std::ffi::OsString]) -> bool {
    matches!(
        tail.first().and_then(|arg| arg.to_str()),
        Some("list" | "info" | "token" | "params" | "proposal" | "proposals" | "neurons")
    )
}
