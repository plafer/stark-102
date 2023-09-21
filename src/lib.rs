#![feature(slice_as_chunks)]

// Questions to answer
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
pub mod field;
pub mod merkle;
pub mod poly;
pub mod prover;
pub mod trace;
pub mod util;

use field::BaseField;
use merkle::{MerklePath, MerkleRoot};

#[derive(Clone, Debug)]
pub struct StarkProof {
    // Commitment phase
    pub trace_lde_commitment: MerkleRoot,
    pub composition_poly_lde_commitment: MerkleRoot,

    // The composition polynomial has degree 7 (it was interpolated on 8
    // points). Hence, the first FRI layer has half that degree, and so on until
    // we're at degree 0.
    pub fri_layer_deg_3_commitment: MerkleRoot,
    pub fri_layer_deg_1_commitment: MerkleRoot,
    pub fri_layer_deg_0_commitment: MerkleRoot,

    // TODO Q: add explicitly the constant element of last layer? They do in Stark 101.

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
}
