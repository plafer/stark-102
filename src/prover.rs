use crate::{
    channel::Channel,
    constraints::composition_polynomial,
    domain::{DOMAIN_LDE, DOMAIN_TRACE},
    field::BaseField,
    merkle::{MerklePath, MerkleTree},
    poly::Polynomial,
    trace::generate_trace,
    ProofQueryPhase, StarkProof,
};

pub fn generate_proof() -> StarkProof {
    let mut channel = Channel::new();

    ////////////////////
    // Commitment phase
    ////////////////////

    // Trace
    let trace = generate_trace();
    let trace_polynomial = Polynomial::lagrange_interp(&DOMAIN_TRACE, &trace).unwrap();

    let trace_lde = trace_polynomial.eval_domain(&DOMAIN_LDE);
    let trace_lde_merkleized = MerkleTree::new(&trace_lde);

    channel.commit(trace_lde_merkleized.root);

    // Composition polynomial
    let cp = {
        let alpha_0 = channel.random_element();
        let alpha_1 = channel.random_element();

        composition_polynomial(alpha_0, alpha_1)
    };

    let cp_lde = cp.eval_domain(&DOMAIN_LDE);
    let cp_lde_merkleized = MerkleTree::new(&cp_lde);

    channel.commit(cp_lde_merkleized.root);

    // TODO: Describe in the README the intuition of the channel. How it can be
    // thought of as an interactive protocol between the prover and verifier,
    // but made non-interactive using the Fiat-Shamir trick.

    // FRI
    let beta_fri_deg_1 = channel.random_element();
    let (domain_deg_1, fri_layer_deg_1_poly) = fri_step(&DOMAIN_LDE, cp.clone(), beta_fri_deg_1);
    let fri_layer_deg_1_eval = fri_layer_deg_1_poly.eval_domain(&domain_deg_1);
    let fri_layer_deg_1_merkleized = MerkleTree::new(&fri_layer_deg_1_eval);

    channel.commit(fri_layer_deg_1_merkleized.root);

    let beta_fri_deg_0 = channel.random_element();
    let (domain_deg_0, fri_layer_deg_0_poly) =
        fri_step(&domain_deg_1, fri_layer_deg_1_poly.clone(), beta_fri_deg_0);

    // The last layer has degree 0, with 2 elements. Therefore, we expect both
    // of these elements to be the same value (a degree 0 polynomial is a
    // constant function, meaning that it evaluates to the same value
    // everywhere).
    assert_eq!(domain_deg_0.len(), 2);
    assert_eq!(
        fri_layer_deg_0_poly.eval(domain_deg_0[0]),
        fri_layer_deg_0_poly.eval(domain_deg_0[1])
    );

    let fri_layer_deg_0_eval = fri_layer_deg_0_poly.eval(domain_deg_0[0]);

    ////////////////////
    // Query phase
    ////////////////////

    // Note: We will need to send (extended) trace elements at index i and i+2.
    // Since our (extended) trace has 8 elements, we draw i to be between [0,
    // 7].
    //
    // Let's see why that is. Let g be the generator of the trace domain (size
    // of 4), and w be the generator of the LDE domain (size of 8). We know g=13
    // and w=9. We notice that g = w^2. Let's say we draw index i, to give us
    // the trace element `t(hw^i)`, where `h=3` is the shift element to give us
    // the coset (see `CyclicGroup`). We want to know the index of `t(g *
    // hw^i)`. We have that `t(ghw^i) = t(w^2 * h * w^i) = t(h * w^(i+2))`, so
    // the index is `i+2`.

    let query_idx = channel.random_integer(8 - 2) as usize;

    let query_phase = generate_query_phase(
        query_idx,
        &trace_lde,
        &trace_lde_merkleized,
        &cp_lde,
        &cp_lde_merkleized,
        &fri_layer_deg_1_eval,
        &fri_layer_deg_1_merkleized,
        fri_layer_deg_0_eval,
    );

    let commitments = channel.finalize();
    assert_eq!(
        commitments.len(),
        3,
        "Expected 3 commitments, got {}",
        commitments.len()
    );

    StarkProof {
        trace_lde_commitment: commitments[0],
        composition_poly_lde_commitment: commitments[1],
        fri_layer_deg_1_commitment: commitments[2],
        query_phase,
    }
}

// Returns the domain and polynomial of the next FRI layer
fn fri_step(
    domain: &[BaseField],
    polynomial: Polynomial,
    beta: BaseField,
) -> (Vec<BaseField>, Polynomial) {
    // The domain of the next FRI layer is (the first or second) half of the
    // current domain, where every element is squared. Both the first or second
    // half squared result in the same domain. For example, given a domain with generator g,
    //
    // dom = {g^0, g^1, g^2, g^3}
    // first_half = {g^0, g^1}
    // first_half_squared = {g^0, g^2}
    //
    // second_half = {g^2, g^3}
    // second_half_squared = {g^4, g^6} = {g^0, g^2}
    // ^ The second equality is true because g^4 = 1 (by definition of g being the generator)
    //
    // Refer to Stark 101 part 3 for more information.
    let next_domain = domain[0..domain.len() / 2]
        .iter()
        .map(|x| x.exp(2))
        .collect();

    (next_domain, polynomial.fri_step(beta))
}

#[allow(clippy::too_many_arguments)]
fn generate_query_phase(
    query_idx: usize,
    trace_lde: &[BaseField],
    trace_lde_merkleized: &MerkleTree,
    cp_lde: &[BaseField],
    cp_lde_merkleized: &MerkleTree,
    fri_layer_deg_1_eval: &[BaseField],
    fri_layer_deg_1_merkleized: &MerkleTree,
    fri_layer_deg_0_eval: BaseField,
) -> ProofQueryPhase {
    let t_x = trace_lde[query_idx];
    let t_x_proof = MerklePath::new(trace_lde_merkleized, query_idx)
        .expect("query index is between 0 and 5, and Merkle tree has 8 elements");

    let t_gx = trace_lde[query_idx + 2];
    let t_gx_proof = MerklePath::new(trace_lde_merkleized, query_idx + 2)
        .expect("query index is between 2 and 7, and Merkle tree has 8 elements");

    // Query composition polynomial (domain size = 8)
    let (cp_minus_x, cp_minus_x_proof) = {
        let query_idx_minus_x = (query_idx + 4) % 8;

        (
            cp_lde[query_idx_minus_x],
            MerklePath::new(cp_lde_merkleized, query_idx_minus_x).unwrap(),
        )
    };

    // Query FRI layer of degree 1 (domain size = 4)
    // TODO: Explain why it's %4
    // Core idea: [a,b,c,d,e,f,g]^2 -> [x,y,z,w,x,y,z,w].
    // e.g. query_idx = 5, then f^2 = y, and query_idx_next = 5%4 = 1 (which is also `y`)
    let query_idx_fri_1_x = query_idx % 4;

    let (fri_layer_deg_1_minus_x, fri_layer_deg_1_minus_x_proof) = {
        let query_idx_fri_1_minus_x = (query_idx_fri_1_x + 2) % 4;

        (
            fri_layer_deg_1_eval[query_idx_fri_1_minus_x],
            MerklePath::new(fri_layer_deg_1_merkleized, query_idx_fri_1_minus_x).unwrap(),
        )
    };

    ProofQueryPhase {
        trace_x: (t_x, t_x_proof),
        trace_gx: (t_gx, t_gx_proof),
        cp_minus_x: (cp_minus_x, cp_minus_x_proof),
        fri_layer_deg_1_minus_x: (fri_layer_deg_1_minus_x, fri_layer_deg_1_minus_x_proof),
        fri_layer_deg_0_x: fri_layer_deg_0_eval,
    }
}
