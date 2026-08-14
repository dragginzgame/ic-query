//! Module: nns::neuron::report::classification
//!
//! Responsibility: define native NNS neuron classifications and their stable labels.
//! Does not own: raw Governance DTOs, report assembly, or text layout.
//! Boundary: retains unrecognized numeric codes instead of collapsing protocol evidence.

use std::fmt;

macro_rules! native_code_classification {
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

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$variant => formatter.write_str($label),)+
                    Self::$unknown(code) => write!(formatter, "unknown({code})"),
                }
            }
        }

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
                deserialize_code_label(
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

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Self::$absent => formatter.write_str("unknown"),
                    $(Self::$variant => formatter.write_str($label),)+
                    Self::$unknown(code) => write!(formatter, "unknown({code})"),
                }
            }
        }

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
                deserialize_code_label(
                    deserializer,
                    $classification,
                    |label| match label {
                        "unknown" => Some(Self::$absent),
                        $($label => Some(Self::$variant),)+
                        _ => None,
                    },
                    Self::$unknown,
                )
            }
        }
    };
}

native_code_classification! {
    "state";
    ///
    /// NnsNeuronState
    ///
    /// Native NNS Governance neuron state with unrecognized codes retained.
    ///
    pub enum NnsNeuronState {
        /// Governance supplied the unspecified state code.
        Unspecified = 0 => "unspecified",
        /// Neuron is not dissolving.
        NotDissolving = 1 => "not-dissolving",
        /// Neuron is dissolving.
        Dissolving = 2 => "dissolving",
        /// Neuron has dissolved.
        Dissolved = 3 => "dissolved",
        /// Neuron is spawning.
        Spawning = 4 => "spawning",
        ; unknown Unknown(i32),
    }
}

native_optional_code_classification! {
    "visibility";
    ///
    /// NnsNeuronVisibility
    ///
    /// Native optional NNS Governance neuron visibility with unknown evidence retained.
    ///
    pub enum NnsNeuronVisibility {
        /// Governance omitted the optional visibility code.
        Unknown,
        /// Governance supplied the unspecified visibility code.
        Unspecified = 0 => "unspecified",
        /// Neuron is private.
        Private = 1 => "private",
        /// Neuron is public.
        Public = 2 => "public",
        ; unknown UnknownCode(i32),
    }
}

native_optional_code_classification! {
    "type";
    ///
    /// NnsNeuronType
    ///
    /// Native optional NNS Governance neuron type with unknown evidence retained.
    ///
    pub enum NnsNeuronType {
        /// Governance omitted the optional neuron-type code.
        Unknown,
        /// Governance supplied the unspecified neuron-type code.
        Unspecified = 0 => "unspecified",
        /// Seed neuron.
        Seed = 1 => "seed",
        /// Early-contributor-token neuron.
        Ect = 2 => "ect",
        ; unknown UnknownCode(i32),
    }
}

native_code_classification! {
    "vote";
    ///
    /// NnsNeuronVote
    ///
    /// Native NNS Governance neuron-ballot vote with unrecognized codes retained.
    ///
    pub enum NnsNeuronVote {
        /// Governance supplied the unspecified vote code.
        Unspecified = 0 => "unspecified",
        /// Affirmative ballot.
        Yes = 1 => "yes",
        /// Negative ballot.
        No = 2 => "no",
        ; unknown Unknown(i32),
    }
}

fn deserialize_code_label<'de, D, T>(
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
    let code = parse_unknown_code(&label).ok_or_else(|| {
        D::Error::custom(format!(
            "invalid NNS neuron {classification} label {label:?}"
        ))
    })?;
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
mod tests {
    use super::{NnsNeuronState, NnsNeuronType, NnsNeuronVisibility, NnsNeuronVote};

    #[test]
    fn required_classifications_preserve_codes_and_labels() {
        for (code, state, label) in [
            (0, NnsNeuronState::Unspecified, "unspecified"),
            (1, NnsNeuronState::NotDissolving, "not-dissolving"),
            (2, NnsNeuronState::Dissolving, "dissolving"),
            (3, NnsNeuronState::Dissolved, "dissolved"),
            (4, NnsNeuronState::Spawning, "spawning"),
            (99, NnsNeuronState::Unknown(99), "unknown(99)"),
        ] {
            assert_eq!(NnsNeuronState::from_code(code), state);
            assert_eq!(state.code(), code);
            assert_eq!(state.to_string(), label);
            assert_eq!(serde_json::to_value(state).expect("serialize state"), label);
        }
        for (code, vote, label) in [
            (0, NnsNeuronVote::Unspecified, "unspecified"),
            (1, NnsNeuronVote::Yes, "yes"),
            (2, NnsNeuronVote::No, "no"),
            (99, NnsNeuronVote::Unknown(99), "unknown(99)"),
        ] {
            assert_eq!(NnsNeuronVote::from_code(code), vote);
            assert_eq!(vote.code(), code);
            assert_eq!(vote.to_string(), label);
            assert_eq!(serde_json::to_value(vote).expect("serialize vote"), label);
        }
    }

    #[test]
    fn optional_classifications_distinguish_absent_and_unknown_codes() {
        for (code, visibility, label) in [
            (None, NnsNeuronVisibility::Unknown, "unknown"),
            (Some(0), NnsNeuronVisibility::Unspecified, "unspecified"),
            (Some(1), NnsNeuronVisibility::Private, "private"),
            (Some(2), NnsNeuronVisibility::Public, "public"),
            (
                Some(99),
                NnsNeuronVisibility::UnknownCode(99),
                "unknown(99)",
            ),
        ] {
            assert_eq!(NnsNeuronVisibility::from_code(code), visibility);
            assert_eq!(visibility.code(), code);
            assert_eq!(visibility.to_string(), label);
            assert_eq!(
                serde_json::to_value(visibility).expect("serialize visibility"),
                label
            );
        }
        for (code, neuron_type, label) in [
            (None, NnsNeuronType::Unknown, "unknown"),
            (Some(0), NnsNeuronType::Unspecified, "unspecified"),
            (Some(1), NnsNeuronType::Seed, "seed"),
            (Some(2), NnsNeuronType::Ect, "ect"),
            (Some(99), NnsNeuronType::UnknownCode(99), "unknown(99)"),
        ] {
            assert_eq!(NnsNeuronType::from_code(code), neuron_type);
            assert_eq!(neuron_type.code(), code);
            assert_eq!(neuron_type.to_string(), label);
            assert_eq!(
                serde_json::to_value(neuron_type).expect("serialize neuron type"),
                label
            );
        }
    }

    #[test]
    fn classifications_read_canonical_cache_labels_only() {
        assert_eq!(
            serde_json::from_str::<NnsNeuronState>("\"unknown(-9)\"")
                .expect("deserialize unknown state"),
            NnsNeuronState::Unknown(-9)
        );
        assert_eq!(
            serde_json::from_str::<NnsNeuronVisibility>("\"unknown\"")
                .expect("deserialize absent visibility"),
            NnsNeuronVisibility::Unknown
        );
        assert_eq!(
            serde_json::from_str::<NnsNeuronType>("\"unknown(9)\"")
                .expect("deserialize unknown neuron type"),
            NnsNeuronType::UnknownCode(9)
        );
        assert_eq!(
            serde_json::from_str::<NnsNeuronVote>("\"yes\"").expect("deserialize known vote"),
            NnsNeuronVote::Yes
        );
        assert!(serde_json::from_str::<NnsNeuronState>("\"unknown(+9)\"").is_err());
        assert!(serde_json::from_str::<NnsNeuronVote>("\"maybe\"").is_err());
    }
}
