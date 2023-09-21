use crate::{
    channel::Channel,
    constraints::composition_polynomial,
    field::{BaseField, CyclicGroup},
    merkle::{MerklePath, MerkleTree},
    poly::Polynomial,
    trace::generate_trace,
    ProofQueryPhase, StarkProof,
};

const CHANNEL_SALT: [u8; 1] = [42u8];

pub fn generate_proof() -> StarkProof {
    let mut channel = Channel::new(&CHANNEL_SALT);

    ////////////////////
    // Commitment phase
    ////////////////////

    // Trace
    let trace = generate_trace();
    let trace_domain = CyclicGroup::new(4).unwrap();
    let trace_polynomial = Polynomial::lagrange_interp(&trace_domain, &trace).unwrap();

    let lde_domain = CyclicGroup::new(8).unwrap();
    let trace_lde = trace_polynomial.eval_domain(&lde_domain.elements);
    let trace_lde_merkleized = MerkleTree::new(&trace_lde);

    channel.commit(trace_lde_merkleized.root);

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

    channel.commit(cp_lde_merkleized.root);

    // TODO: Describe in the README the intuition of the channel. How it can be
    // thought of as an interactive protocol between the prover and verifier,
    // but made non-interactive using the Fiat-Shamir trick.

    // FRI
    let beta_fri_deg_3 = channel.random_element();
    let (domain_deg_3, fri_layer_deg_3) =
        fri_step(&lde_domain.elements, cp.clone(), beta_fri_deg_3);
    let fri_layer_deg_3_merkleized = MerkleTree::new(&fri_layer_deg_3.eval_domain(&domain_deg_3));

    channel.commit(fri_layer_deg_3_merkleized.root);

    let beta_fri_deg_1 = channel.random_element();
    let (domain_deg_1, fri_layer_deg_1) =
        fri_step(&domain_deg_3, fri_layer_deg_3.clone(), beta_fri_deg_1);
    let fri_layer_deg_1_merkleized = MerkleTree::new(&fri_layer_deg_1.eval_domain(&domain_deg_1));

    channel.commit(fri_layer_deg_1_merkleized.root);

    let beta_fri_deg_0 = channel.random_element();
    let (domain_deg_0, fri_layer_deg_0) =
        fri_step(&domain_deg_1, fri_layer_deg_1.clone(), beta_fri_deg_0);
    let fri_layer_deg_0_merkleized = MerkleTree::new(&fri_layer_deg_0.eval_domain(&domain_deg_0));

    channel.commit(fri_layer_deg_0_merkleized.root);

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

    let t_x = trace_lde[query_idx];
    let t_x_proof = MerklePath::new(&trace_lde_merkleized, query_idx)
        .expect("query index is between 0 and 5, and Merkle tree has 8 elements");

    let t_gx = trace_lde[query_idx + 2];
    let t_gx_proof = MerklePath::new(&trace_lde_merkleized, query_idx + 2)
        .expect("query index is between 2 and 7, and Merkle tree has 8 elements");

    let commitments = channel.finalize();
    assert_eq!(
        commitments.len(),
        5,
        "Expected 5 commitments; did we forget to commit a value somewhere?"
    );

    StarkProof {
        trace_lde_commitment: commitments[0],
        composition_poly_lde_commitment: commitments[1],
        fri_layer_deg_3_commitment: commitments[2],
        fri_layer_deg_1_commitment: commitments[3],
        fri_layer_deg_0_commitment: commitments[4],
        query_phase: ProofQueryPhase {
            trace_x: (t_x, t_x_proof),
            trace_gx: (t_gx, t_gx_proof),
        },
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
