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
mod tests;
