use std::ops::Index;

use crate::field::BaseField;

pub static DOMAIN_TRACE: DomainTrace = DomainTrace {
    elements: [
        BaseField::new(1),
        BaseField::new(13),
        BaseField::new(16),
        BaseField::new(4),
    ],
};

pub struct DomainTrace {
    elements: [BaseField; 4],
}

impl DomainTrace {
    pub const fn generator() -> BaseField {
        BaseField::new(13)
    }
}

impl Index<usize> for DomainTrace {
    type Output = BaseField;

    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}
