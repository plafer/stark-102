use std::{ops::{Mul, Div, Add}, fmt::Display};

use anyhow::bail;

const PRIME: u8 = 17;

/// Represents an element of the prime field with prime 17.
/// This group contains a multiplicative group of 16 elements,
/// and cyclic subgroups of size 4 and 8.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaseField {
    element: u8,
}

impl BaseField {
    pub fn new(element: u8) -> Self {
        Self {
            element: element % PRIME,
        }
    }

    pub fn square(&self) -> Self {
        Self {
            element: (self.element * self.element) % PRIME,
        }
    }
}

impl From<u8> for BaseField {
    fn from(element: u8) -> Self {
        Self::new(element)
    }
}

impl Add for BaseField {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            element: (self.element + rhs.element) % PRIME,
        }
    }
}

impl Mul for BaseField {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            element: (self.element * rhs.element) % PRIME,
        }
    }
}

impl Div for BaseField {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.element % rhs.element != 0 {
            panic!("Division of {self} and {rhs} has nonzero remainder");
        }

        Self {
            element: self.element / rhs.element
        }
    }
}

impl Display for BaseField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.element)
    }
}

/// Describes a cyclic group/subgroup of BaseField
pub struct CyclicGroup {
    pub generator: BaseField,
    pub elements: Vec<BaseField>,
}

impl CyclicGroup {
    pub fn new(size: u8) -> anyhow::Result<Self> {
        if size != 4 || size != 8 {
            bail!("Unsupported group size: {size}")
        }

        if size == 4 {
            Ok(Self {
                generator: BaseField::new(13),
                elements: vec![1.into(), 13.into(), 16.into(), 4.into()],
            })
        } else {
            todo!()
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}
