use std::ops::{Deref, Index};

use crate::field::BaseField;

/// Represents the domain of the trace polynomial. That is, when we interpolate
/// a polynomial over the trace, we use `DOMAIN_TRACE` as the domain of the
/// interpolated polynomial.
pub static DOMAIN_TRACE: Domain<4, 13> = Domain {
    elements: [
        BaseField::new(1),
        BaseField::new(13),
        BaseField::new(16),
        BaseField::new(4),
    ],
};

/// Represents the domain of the low-degree extended (LDE) trace. This domain
/// was constructed conceptually in 2 steps:
///
/// 1. Take the multiplicative subgroup of size 8 of `BaseField` (technically
///    `BaseField` without `0`). This turns out to be the group [1, 9, 13, 15,
///    16, 8, 4, 2] with generator 9.
///
///    The problem with simply using the above subgroup is that it shares 1, 4
///    and 13 with `DOMAIN_TRACE`. If the verifier were to query the LDE trace
///    at any of these positions, it would be reading some original data points.
///    This would make the "STARK" not zero-knowledge, since the verifier would
///    be able to read some private data. Note that this doesn't apply to our
///    specific problem, since the verifier can easily compute the sequence 3,
///    3^2, 3^4, 3^8 for themselves. However, some problems do, such as if we
///    were proving the statement "I know x such that SHA256(x) = <some hash>".
///    Then the first element of the trace would be the private `x`, and a query
///    `trace_lde(1) = x` would leak the `x`.
///
/// 2. Group theory tells us that we can expect the multiplicative group {1,
///    ..., 16} to be decomposed into 2 disjoint subgroups. From step 1, we know
///    one group. If we multiply every element by an element of the coset, then
///    we're guaranteed to get another subgroup of {1, ..., 16}, disjoint from
///    the one in step 1 (called the *coset* of the group in step 1). We choose
///    3.
///
/// You can verify yourself that `DOMAIN_LDE` is a multiplicative group, and is
/// disjoint from the group in step 1.
pub static DOMAIN_LDE: Domain<8, 9> = Domain {
    elements: [
        BaseField::new(3),
        BaseField::new(10),
        BaseField::new(5),
        BaseField::new(11),
        BaseField::new(14),
        BaseField::new(7),
        BaseField::new(12),
        BaseField::new(6),
    ],
};

/// Represents the domain of either the trace polynomial (see `DOMAIN_TRACE`) or
/// the LDE trace polynomial (see `DOMAIN_LDE`).
///
/// Both domains are cyclic groups; the `GENERATOR` const generic is the value
/// of the group generator. `N` is the size of the domain.
pub struct Domain<const N: usize, const GENERATOR: u8> {
    elements: [BaseField; N],
}

impl<const N: usize, const GENERATOR: u8> Domain<N, GENERATOR> {
    pub const fn generator() -> BaseField {
        BaseField::new(GENERATOR)
    }
}

impl<const N: usize, const GENERATOR: u8> Index<usize> for Domain<N, GENERATOR> {
    type Output = BaseField;

    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}

impl<const N: usize, const GENERATOR: u8> Deref for Domain<N, GENERATOR> {
    type Target = [BaseField];

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}
