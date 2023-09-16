use crate::{merkle::MerkleTree, trace::generate_trace, StarkProof};

pub fn generate_proof() -> StarkProof {
    let trace_merkleized = trace();

    StarkProof {
        trace_commitment: trace_merkleized.root,
    }
}

fn trace() -> MerkleTree {
    let trace = generate_trace();

    MerkleTree::new(&trace)
}
