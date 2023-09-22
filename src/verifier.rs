use crate::{channel::Channel, StarkProof};

pub fn verify(proof: &StarkProof) -> anyhow::Result<()> {
    let mut channel = Channel::new();

    // We interact with the channel in the exact same way the prover does, in
    // order to draw the same values the prover did when generating the proof.
    channel.commit(proof.trace_lde_commitment);

    let alpha_0 = channel.random_element();
    let alpha_1 = channel.random_element();

    channel.commit(proof.composition_poly_lde_commitment);

    let beta_fri_deg_3 = channel.random_element();
    channel.commit(proof.fri_layer_deg_3_commitment);

    let beta_fri_deg_1 = channel.random_element();
    channel.commit(proof.fri_layer_deg_1_commitment);

    let beta_fri_deg_0 = channel.random_element();

    let query_idx = channel.random_integer(8 - 2) as usize;

    // Next, verify all the Merkle proofs, to make sure that whatever is in the
    // proof struct is valid.

    Ok(())
}
