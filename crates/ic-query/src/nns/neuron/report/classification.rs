//! Module: nns::neuron::report::classification
//!
//! Responsibility: define native NNS neuron classifications and their stable labels.
//! Does not own: raw Governance DTOs, report assembly, or text layout.
//! Boundary: retains unrecognized numeric codes instead of collapsing protocol evidence.

use crate::nns::governance::{native_code_classification, native_optional_code_classification};

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

#[cfg(test)]
mod tests {
    use super::{NnsNeuronState, NnsNeuronType, NnsNeuronVisibility};

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
        assert!(serde_json::from_str::<NnsNeuronState>("\"unknown(+9)\"").is_err());
        assert!(
            serde_json::from_str::<crate::nns::governance::NnsGovernanceVote>("\"maybe\"").is_err()
        );
    }
}
