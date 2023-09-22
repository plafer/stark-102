use crate::{field::BaseField, poly::Polynomial};

/// TODO: Make the `3` a public parameter, so that we can have a trace param here
///
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

/// Note that we construct our composition polynomial as they do in Stark 101
/// (i.e. by taking a random linear combination of the boundary and transition
/// constraint polynomials) as opposed to what they do in
/// [the lambdaclass blog post](https://blog.lambdaclass.com/diving-deep-fri#the-constraint-composition-polynomial)
pub fn composition_polynomial(alpha_0: BaseField, alpha_1: BaseField) -> Polynomial {
    let mut p0 = boundary_constraint();
    p0.scalar_mul(alpha_0);

    let mut p1 = transition_constraint();
    p1.scalar_mul(alpha_1);

    p0 + p1
}
