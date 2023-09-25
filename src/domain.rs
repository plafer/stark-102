use std::ops::{Deref, Index};

use crate::field::BaseField;

/// TODO: Represents domain trace, and what that is
pub static DOMAIN_TRACE: Domain<4, 13> = Domain {
    elements: [
        BaseField::new(1),
        BaseField::new(13),
        BaseField::new(16),
        BaseField::new(4),
    ],
};

/// TODO: How we get that domain (coset)
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

/// TODO: Document generic params
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
