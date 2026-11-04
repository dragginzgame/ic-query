//! Shared native NNS classification codecs and ballot votes.

macro_rules! native_code_classification {
    (@codec $name:ident, $classification:literal, $unknown:ident, {
        $($label:literal => $variant:ident,)+
    }) => {
        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.collect_str(self)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::nns::governance::deserialize_code_label(
                    deserializer,
                    $classification,
                    |label| match label {
                        $($label => Some(Self::$variant),)+
                        _ => None,
                    },
                    Self::$unknown,
                )
            }
        }
    };
    (
        $classification:literal;
        $(#[$enum_meta:meta])*
        pub enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $code:literal => $label:literal,
            )+
            ; unknown $unknown:ident(i32),
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
            /// Governance supplied an unrecognized native code.
            $unknown(i32),
        }

        impl $name {
            /// Classify one raw native code without discarding unknown evidence.
            #[must_use]
            pub const fn from_code(code: i32) -> Self {
                match code {
                    $($code => Self::$variant,)+
                    code => Self::$unknown(code),
                }
            }

            /// Return the exact native code represented by this classification.
            #[must_use]
            pub const fn code(self) -> i32 {
                match self {
                    $(Self::$variant => $code,)+
                    Self::$unknown(code) => code,
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant => formatter.write_str($label),)+
                    Self::$unknown(code) => write!(formatter, "unknown({code})"),
                }
            }
        }

        $crate::nns::governance::native_code_classification! {
            @codec $name, $classification, $unknown, {
                $($label => $variant,)+
            }
        }

    };
}

macro_rules! native_optional_code_classification {
    (
        $classification:literal;
        $(#[$enum_meta:meta])*
        pub enum $name:ident {
            $(#[$absent_meta:meta])*
            $absent:ident,
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $code:literal => $label:literal,
            )+
            ; unknown $unknown:ident(i32),
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $(#[$absent_meta])*
            $absent,
            $(
                $(#[$variant_meta])*
                $variant,
            )+
            /// Governance supplied an unrecognized native code.
            $unknown(i32),
        }

        impl $name {
            /// Classify one optional raw native code without discarding unknown evidence.
            #[must_use]
            pub const fn from_code(code: Option<i32>) -> Self {
                match code {
                    None => Self::$absent,
                    $(Some($code) => Self::$variant,)+
                    Some(code) => Self::$unknown(code),
                }
            }

            /// Return the exact optional native code represented by this classification.
            #[must_use]
            pub const fn code(self) -> Option<i32> {
                match self {
                    Self::$absent => None,
                    $(Self::$variant => Some($code),)+
                    Self::$unknown(code) => Some(code),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::$absent => formatter.write_str("unknown"),
                    $(Self::$variant => formatter.write_str($label),)+
                    Self::$unknown(code) => write!(formatter, "unknown({code})"),
                }
            }
        }

        $crate::nns::governance::native_code_classification! {
            @codec $name, $classification, $unknown, {
                "unknown" => $absent,
                $($label => $variant,)+
            }
        }

    };
}

pub(in crate::nns) use {native_code_classification, native_optional_code_classification};

native_code_classification! {
    "vote";
    ///
    /// NnsGovernanceVote
    ///
    /// Native NNS Governance ballot vote with unrecognized codes retained.
    ///

    pub enum NnsGovernanceVote {
        /// Governance supplied the unspecified vote code.
        Unspecified = 0 => "unspecified",
        /// Affirmative ballot.
        Yes = 1 => "yes",
        /// Negative ballot.
        No = 2 => "no",
        ; unknown Unknown(i32),
    }
}

pub(in crate::nns) fn deserialize_code_label<'de, D, T>(
    deserializer: D,
    classification: &str,
    known: fn(&str) -> Option<T>,
    unknown: fn(i32) -> T,
) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;

    let label = <String as serde::Deserialize>::deserialize(deserializer)?;
    if let Some(value) = known(&label) {
        return Ok(value);
    }
    let code = parse_unknown_code(&label)
        .ok_or_else(|| D::Error::custom(format!("invalid NNS {classification} label {label:?}")))?;
    Ok(unknown(code))
}

fn parse_unknown_code(label: &str) -> Option<i32> {
    let code = label
        .strip_prefix("unknown(")?
        .strip_suffix(')')?
        .parse::<i32>()
        .ok()?;
    (label == format!("unknown({code})")).then_some(code)
}

#[cfg(test)]
mod tests;
