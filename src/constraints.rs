use crate::{field::BaseField, poly::Polynomial};

/// Polynomial representation of our boundary constraint that the first element
/// of the trace is 3; that is, t(1) = 3. This gets converted into a statement
/// of the form "<some expression agreed to by the prover and verifier> is a
/// polynomial". We prove that by constructing the polynomial, and proving that
/// we have it using FRI.
///
/// Note that we were able to derive the polynomial by hand because this library
/// only cares about this problem (i.e. this boundary constraint). In a more
/// general system like winterfell, we would need to programatically derive the
/// polynomial.
fn boundary_constraint() -> Polynomial {
    Polynomial::new(vec![14.into(), 15.into(), 13.into()])
}

/// This polynomial encodes the transition constraints that check that for the
/// first 3 elements `x` of the trace, the next is equal to `x^2`.
fn transition_constraint() -> Polynomial {
    Polynomial::new(vec![16.into(), 9.into(), 12.into(), 1.into()])
}

/// For definition of alphas and betas, refer to
/// [here](https://blog.lambdaclass.com/diving-deep-fri#the-constraint-composition-polynomial)
pub fn composition_polynomial(
    alpha_1: BaseField,
    beta_1: BaseField,
    alpha_2: BaseField,
    beta_2: BaseField,
) -> Polynomial {
    // alpha_1 * x^2 + beta_1
    let random_poly_boundary = Polynomial::new(vec![beta_1, 0.into(), alpha_1]);

    // alpha_2 * x + beta_2
    let random_poly_transition = Polynomial::new(vec![beta_2, alpha_2]);

    (boundary_constraint() * random_poly_boundary)
        + (transition_constraint() * random_poly_transition)
}
