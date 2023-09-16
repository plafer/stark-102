use crate::{
    field::CyclicGroup, merkle::MerkleTree, poly::Polynomial, trace::generate_trace, StarkProof,
};

pub fn generate_proof() -> StarkProof {
    let trace = generate_trace();
    let trace_domain = CyclicGroup::new(4).unwrap();
    let trace_polynomial = Polynomial::lagrange_interp(&trace_domain, &trace).unwrap();

    let lde_domain = CyclicGroup::new(8).unwrap();
    let trace_lde = trace_polynomial.eval_domain(&lde_domain.elements);
    let trace_lde_merkleized = MerkleTree::new(&trace_lde);

    StarkProof {
        trace_lde_commitment: trace_lde_merkleized.root,
    }
}
