//! Module: cli
//!
//! Responsibility: shared CLI parsing, help, and output utilities.
//! Does not own: NNS/SNS command execution or report construction.
//! Boundary: exposes command-family helpers and clap wrappers used by command modules.

pub mod clap;
pub mod common;

macro_rules! value_enum_with_conversion {
    (
        $(#[$enum_meta:meta])*
        $visibility:vis enum $name:ident => $target:ty {
            $($(#[$variant_meta:meta])* $variant:ident,)+
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, clap::ValueEnum)]
        $visibility enum $name {
            $($(#[$variant_meta])* $variant,)+
        }

        impl From<$name> for $target {
            fn from(value: $name) -> Self {
                match value {
                    $($name::$variant => Self::$variant,)+
                }
            }
        }
    };
}

pub(crate) use value_enum_with_conversion;

#[cfg(test)]
pub use tests::parse_test_args;
#[cfg(test)]
mod tests;
