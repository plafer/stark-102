#![feature(slice_as_chunks)]

pub mod channel;
pub mod constraints;
pub mod domain;
pub mod field;
pub mod merkle;
pub mod poly;
pub(crate) mod prover;
pub mod trace;
pub mod util;
pub(crate) mod verifier;

use field::BaseField;
use merkle::{MerklePath, MerkleRoot};

/// Generate the STARK
pub use prover::generate_proof;

/// Verify the STARK
pub use verifier::verify;

#[derive(Clone, Debug)]
pub struct StarkProof {
    // Commitment phase
    pub trace_lde_commitment: MerkleRoot,

    // The composition polynomial has degree 3 (it was *interpolated* on 4
    // points, and *evaluated* on 8).
    pub composition_poly_lde_commitment: MerkleRoot,

    // The first FRI layer has half the degree of the composition polynomial
    // (i.e. degree 1)
    pub fri_layer_deg_1_commitment: MerkleRoot,

    pub query_phase: ProofQueryPhase,
}

/// Our STARK proof only supports one query. However, in production systems, we
/// want to do more than one query to increase the security of the system.
#[derive(Clone, Debug)]
pub struct ProofQueryPhase {
    pub trace_x: (BaseField, MerklePath),

    // trace(gx); where g is the generator for the original domain (size 4)
    pub trace_gx: (BaseField, MerklePath),

    // `composition_polynomial(-x)` (degree 3)
    pub cp_minus_x: (BaseField, MerklePath),

    // fri_layer_deg_1_eval(-x^2)
    pub fri_layer_deg_1_minus_x: (BaseField, MerklePath),

    // fri_layer_deg_0_eval(x^4)
    pub fri_layer_deg_0_x: BaseField,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn proof_verification() {
        let proof = generate_proof();
        let verify_result = verify(&proof);

        assert!(verify_result.is_ok(), "Error: {verify_result:?}");
    }
}
