use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, Mul, Sub, MulAssign, DivAssign},
};

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
    // FIXME: It would be better to accept i32 here, or i64
    pub fn new(element: u8) -> Self {
        Self {
            element: element % PRIME,
        }
    }

    pub fn zero() -> Self {
        Self { element: 0u8 }
    }

    pub fn one() -> Self {
        Self { element: 1u8 }
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

impl From<i32> for BaseField {
    fn from(num: i32) -> Self {
        // Note: We do this because e.g. -1 % 17 = -1.
        // We then instead do 16 % 17 = 16
        let adjusted_num = num + PRIME as i32;
        Self::new((adjusted_num % (PRIME as i32)) as u8)
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

impl AddAssign for BaseField {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for BaseField {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            element: ((self.element + PRIME) - rhs.element) % PRIME,
        }
    }
}

impl Mul for BaseField {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // We need this trick because 16 * 16 = 256 and overflows the u8.
        let mul_minus_one = self.element * (rhs.element - 1u8) % PRIME;
        Self {
            element: (mul_minus_one + self.element) % PRIME,
        }
    }
}

impl MulAssign for BaseField {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Div for BaseField {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.element % rhs.element != 0 {
            panic!("Division of {self} and {rhs} has nonzero remainder");
        }

        Self {
            element: self.element / rhs.element,
        }
    }
}

impl DivAssign for BaseField {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
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
        if size != 4 && size != 8 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_i32() {
        let ele = BaseField::from(-1);

        assert_eq!(ele, BaseField::new(16u8));
    }

    #[test]
    fn test_mul() {
        assert_eq!(BaseField::from(1) * BaseField::from(1), BaseField::from(1));
        assert_eq!(BaseField::from(100) * BaseField::from(100), BaseField::from(4));
        // This overflows the u8 if we're not careful
        assert_eq!(BaseField::from(16) * BaseField::from(16), BaseField::from(1));
    }

    #[test]
    fn test_sub() {
        assert_eq!(BaseField::from(1) - BaseField::from(2), BaseField::from(16));
        assert_eq!(BaseField::from(16) - BaseField::from(2), BaseField::from(14));
        assert_eq!(BaseField::from(16) - BaseField::from(16), BaseField::from(0));
    }
}
