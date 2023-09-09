use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub},
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

    /// Returns the multiplicative inverse for elements in the subgroup {1, ...,
    /// 16} Yes, this is a bit messy, because not all elements of the field have
    /// a multiplicative inverse.
    pub fn mult_inv(&self) -> Self {
        assert!(
            *self != Self::zero(),
            "0 is not in the multiplicative group and has no inverse"
        );

        // The generators of the multiplicative group {1, ..., 16} are
        // 3, 5, 6, 7, 10, 11, 12, 14
        // x/y = x * y^-1, where y * y^-1 = 1
        // For any generator g, say y = g^i for some i. Then y^-1 = g^(16-i).

        let generator = Self::from(3);
        let i = Self::log(*self, generator);

        generator.exp((PRIME - 1) - i)
    }

    pub fn exp(self, exponent: u8) -> Self {
        let mut result = Self::one();

        for _ in 0..exponent {
            result *= self;
        }

        return result;
    }

    /// Computes log_{base}(x); or,
    /// finds i s.t. base**i == x
    ///
    /// Note: by the Discrete Logarithm Problem, we don't know how to
    /// compute this efficiently!
    pub fn log(x: Self, base: Self) -> u8 {
        if x == Self::zero() {
            panic!("log(0) is undefined");
        }
        if x == Self::one() {
            return 0;
        }

        let mut result = Self::one();

        for i in 1..PRIME {
            result *= base;
            if result == x {
                return i;
            }
        }

        panic!("log({x}, {base}) doesn't exist");
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
        if rhs == Self::zero() {
            panic!("Divide by zero")
        }
        if self == Self::zero() {
            return self;
        }

        self * rhs.mult_inv()
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
        assert_eq!(
            BaseField::from(100) * BaseField::from(100),
            BaseField::from(4)
        );
        // This overflows the u8 if we're not careful
        assert_eq!(
            BaseField::from(16) * BaseField::from(16),
            BaseField::from(1)
        );
    }

    #[test]
    fn test_div() {
        for i in 1..PRIME {
            for j in 1..PRIME {
                let numerator = BaseField::from(i);
                let divisor = BaseField::from(j);
                assert_eq!((numerator / divisor) * divisor, numerator,);
            }
        }
    }

    #[test]
    fn test_sub() {
        assert_eq!(BaseField::from(1) - BaseField::from(2), BaseField::from(16));
        assert_eq!(
            BaseField::from(16) - BaseField::from(2),
            BaseField::from(14)
        );
        assert_eq!(
            BaseField::from(16) - BaseField::from(16),
            BaseField::from(0)
        );
    }

    #[test]
    fn test_exp() {
        let field = BaseField::from(4);

        assert_eq!(field.exp(0u8), BaseField::one());
        assert_eq!(field.exp(1u8), field);
        assert_eq!(field.exp(2u8), field * field);

        // By Fermat's Little Theorem
        assert_eq!(field.exp(PRIME - 1), BaseField::one());
    }

    #[test]
    fn test_inv() {
        for i in 1..PRIME {
            let fel = BaseField::from(i);

            assert_eq!(BaseField::one(), fel * fel.mult_inv());
        }
    }
}
