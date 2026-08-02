mod commands;
mod options;
mod run;
#[cfg(test)]
pub(in crate::nns) use commands::{registry_command, registry_version_command};
#[cfg(test)]
pub(in crate::nns) use options::RegistryVersionOptions;
pub(super) use run::{command, run};
