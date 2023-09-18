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

use merkle::MerkleRoot;

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
    // Query phase
}
