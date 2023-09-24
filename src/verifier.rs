use anyhow::bail;

use crate::{
    channel::Channel,
    field::{BaseField, CyclicGroup},
    ProofQueryPhase, StarkProof,
};

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

    verify_query(
        &stark_proof.query_phase,
        alpha_0,
        alpha_1,
        beta_fri_deg_1,
        beta_fri_deg_0,
        query_idx,
    )
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

fn verify_query(
    queries: &ProofQueryPhase,
    alpha_0: BaseField,
    alpha_1: BaseField,
    beta_fri_deg_1: BaseField,
    beta_fri_deg_0: BaseField,
    query_idx: usize,
) -> anyhow::Result<()> {
    let domain_lde = CyclicGroup::new(8)?;
    let x = domain_lde.elements[query_idx];

    // Ensure that the composition polynomial value is actually derived from the trace
    let boundary_constraint_x: BaseField = {
        // FIXME: Don't hardcode `3` and `1`. Make it explicit in code what they are.

        // `3` is the first value of the trace; this is part of the problem
        // definition, and agreed upon by the prover and verifier
        let p1_x = queries.trace_x.0 - BaseField::from(3);

        // `1` is the first value of the original domain.
        p1_x / (x - 1.into())
    };

    let transition_constraint_x: BaseField = {
        let p2_x = queries.trace_gx.0 - queries.trace_x.0.exp(2);

        let denom = (x - 1.into()) * (x - 13.into()) * (x - 16.into());

        p2_x / denom
    };

    // composition_polynomial(x)
    let cp_x = boundary_constraint_x * alpha_0 + transition_constraint_x * alpha_1;

    // FRI layer deg 1
    let fri_layer_deg_1_x: BaseField = {
        let cp_minus_x = queries.cp_minus_x.0;

        let g_x_squared = (cp_x + cp_minus_x) / BaseField::new(2);
        let h_x_squared = (cp_x - cp_minus_x) / (BaseField::new(2) * x);

        g_x_squared + beta_fri_deg_1 * h_x_squared
    };

    // FRI layer deg 0
    let x = x.exp(2);

    let expected_fri_layer_deg_0_x: BaseField = {
        let fri_layer_deg_1_minus_x = queries.fri_layer_deg_1_minus_x.0;

        let g_x_squared = (fri_layer_deg_1_x + fri_layer_deg_1_minus_x) / BaseField::new(2);
        let h_x_squared =
            (fri_layer_deg_1_x - fri_layer_deg_1_minus_x) / (BaseField::new(2) * x);

        g_x_squared + beta_fri_deg_0 * h_x_squared
    };

    if expected_fri_layer_deg_0_x == queries.fri_layer_deg_0_x {
        Ok(())
    } else {
        bail!(
            "Final FRI layer check failed. Value in proof: {}, but computed {}",
            queries.fri_layer_deg_0_x,
            expected_fri_layer_deg_0_x
        )
    }
}
