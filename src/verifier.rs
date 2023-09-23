use anyhow::bail;

use crate::{channel::Channel, StarkProof};

pub fn verify(stark_proof: &StarkProof) -> anyhow::Result<()> {
    let mut channel = Channel::new();

    // We interact with the channel in the exact same way the prover does, in
    // order to draw the same values the prover did when generating the proof.
    channel.commit(stark_proof.trace_lde_commitment);

    let alpha_0 = channel.random_element();
    let alpha_1 = channel.random_element();

    channel.commit(stark_proof.composition_poly_lde_commitment);

    let beta_fri_deg_1 = channel.random_element();
    channel.commit(stark_proof.fri_layer_deg_1_commitment);

    let beta_fri_deg_0 = channel.random_element();

    let query_idx = channel.random_integer(8 - 2) as usize;

    // Verify all the Merkle proofs, to make sure that values in the proof
    // struct are valid.
    verify_merkle_proofs(stark_proof)?;

    // TODO: verify query

    todo!()
}

fn verify_merkle_proofs(stark_proof: &StarkProof) -> anyhow::Result<()> {
    // trace(x)
    {
        let (value, merkle_proof) = &stark_proof.query_phase.trace_x;
        let root = stark_proof.trace_lde_commitment;
        if !merkle_proof.verify_inclusion(*value, root) {
            bail!("trace_x merkle proof verification failed");
        }
    }

    // trace(gx)
    {
        let (value, merkle_proof) = &stark_proof.query_phase.trace_gx;
        let root = stark_proof.trace_lde_commitment;
        if !merkle_proof.verify_inclusion(*value, root) {
            bail!("trace_gx merkle proof verification failed");
        }
    }

    // cp(x)
    {
        let (value, merkle_proof) = &stark_proof.query_phase.cp_x;
        let root = stark_proof.composition_poly_lde_commitment;
        if !merkle_proof.verify_inclusion(*value, root) {
            bail!("cp_x merkle proof verification failed");
        }
    }

    // cp(-x)
    {
        let (value, merkle_proof) = &stark_proof.query_phase.cp_minus_x;
        let root = stark_proof.composition_poly_lde_commitment;
        if !merkle_proof.verify_inclusion(*value, root) {
            bail!("cp_minus_x merkle proof verification failed");
        }
    }

    // FRI layer degree 1 at x^2
    {
        let (value, merkle_proof) = &stark_proof.query_phase.fri_layer_deg_1_x;
        let root = stark_proof.fri_layer_deg_1_commitment;
        if !merkle_proof.verify_inclusion(*value, root) {
            bail!("fri_layer_deg_1_x merkle proof verification failed");
        }
    }

    // FRI layer degree 1 at -x^2
    {
        let (value, merkle_proof) = &stark_proof.query_phase.fri_layer_deg_1_minus_x;
        let root = stark_proof.fri_layer_deg_1_commitment;
        if !merkle_proof.verify_inclusion(*value, root) {
            bail!("fri_layer_deg_1_minus_x merkle proof verification failed");
        }
    }

    Ok(())
}
