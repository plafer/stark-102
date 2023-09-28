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

    // The first FRI layer has half the degree of the composition polynomial;
    // that is, degree 1. The last layer (degree 0) has 2 elements that have the
    // same value (remember: a degree 0 polynomial is a constant function `f(x)
    // = c`). We don't build a Merkle tree for that layer as it is unnecessary:
    // assuming that the prover sends the right value to the verifier (if it
    // doesn't the proof fails anyway), then the Merkle root is deterministic
    // and doesn't provide any new information. (TODO: refine explanation in
    // README)
    pub fri_layer_deg_1_commitment: MerkleRoot,

    pub query_phase: ProofQueryPhase,
}

/// Our STARK proof only supports one query. However, in production systems, we
/// want to do more than one query to increase the security of the system.
#[derive(Clone, Debug)]
pub struct ProofQueryPhase {
    // TODO Q: How does verifier ensure that this is the queried element (i.e. at the
    // expected index)?
    // A: When it will check that CP(x) is as expected, it will use x = g^idx. So if the prover sent the wrong t(x), the verifier's check will fail.
    pub trace_x: (BaseField, MerklePath),

    // trace(gx); where g is the generator for the original domain (size 4)
    pub trace_gx: (BaseField, MerklePath),

    // `composition_polynomial(-x)` (degree 3)
    pub cp_minus_x: (BaseField, MerklePath),

    // fri_layer_deg_1_eval(-x^4)
    pub fri_layer_deg_1_minus_x: (BaseField, MerklePath),

    // fri_layer_deg_0_eval(x^8)
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
