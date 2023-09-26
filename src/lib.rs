#![feature(slice_as_chunks)]

// Questions to answer
// TODO: MOVE TO README
// (t(x) is the trace polynomial)

// Q: How does the verifier ensure the polynomial is correct, even though it's
// evaluated on a disjoint domain?
//
// A: The original statements were translated into statements about polynomials,
// e.g. instead of bd constraint `t(1) = 3`, we have "`t(x)-3/x-1` is a poly".
// And for the proof, the prover will evaluate `t(x)` for `x` in a coset, s.t.
// t(x) never needs to be revealed

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
    // Note: To check that CP(x) is as expected, the verifier needs a separate function to rebuild the CP. That is, it shouldn't cancel out anything. It should evaluate the "long form" of the boundary and transition constraints, and make sure everything checks out.
    pub trace_x: (BaseField, MerklePath),

    // trace(gx); where g is the generator for the original domain (size 4)
    pub trace_gx: (BaseField, MerklePath),

    // `composition_polynomial(-x)` (degree 3)
    pub cp_minus_x: (BaseField, MerklePath),

    // FIXME: Stark 101 sends these values in the channel. Is this necessary?
    // Does winterfell do that? Why/why not?

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
