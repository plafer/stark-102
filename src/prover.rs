use crate::{
    channel::Channel, constraints::composition_polynomial, field::CyclicGroup, merkle::MerkleTree,
    poly::Polynomial, trace::generate_trace, StarkProof,
};

const CHANNEL_SALT: [u8; 1] = [42u8];

pub fn generate_proof() -> StarkProof {
    // Trace
    let trace = generate_trace();
    let trace_domain = CyclicGroup::new(4).unwrap();
    let trace_polynomial = Polynomial::lagrange_interp(&trace_domain, &trace).unwrap();

    let lde_domain = CyclicGroup::new(8).unwrap();
    let trace_lde = trace_polynomial.eval_domain(&lde_domain.elements);
    let trace_lde_merkleized = MerkleTree::new(&trace_lde);

    let mut channel = Channel::new(&CHANNEL_SALT);

    // Composition polynomial
    let cp = {
        let alpha_1 = channel.random_element();
        let beta_1 = channel.random_element();
        let alpha_2 = channel.random_element();
        let beta_2 = channel.random_element();

        composition_polynomial(alpha_1, beta_1, alpha_2, beta_2)
    };

    let cp_lde = cp.eval_domain(&lde_domain.elements);
    let cp_lde_merkleized = MerkleTree::new(&cp_lde);

    StarkProof {
        trace_lde_commitment: trace_lde_merkleized.root,
        composition_poly_lde_commitment: cp_lde_merkleized.root,
    }
}
