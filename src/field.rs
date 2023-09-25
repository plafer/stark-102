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
    pub const fn new(element: u8) -> Self {
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

    /// Returns the multiplicative inverse for elements in the subgroup
    /// {1, ..., 16}
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

    /// Computes the additive inverse (i.e. -x).
    pub fn minus(&self) -> Self {
        BaseField::from(-1) * *self
    }

    pub fn exp(self, exponent: u8) -> Self {
        let mut result = Self::one();

        for _ in 0..exponent {
            result *= self;
        }

        result
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

    pub fn as_byte(&self) -> u8 {
        self.element
    }
}

impl From<u8> for BaseField {
    fn from(element: u8) -> Self {
        Self {
            element: element % PRIME,
        }
    }
}

impl From<BaseField> for u8 {
    fn from(field: BaseField) -> Self {
        field.element
    }
}

impl From<i32> for BaseField {
    fn from(num: i32) -> Self {
        // Note: We do this because e.g. -1 % 17 = -1.
        // We then instead do 16 % 17 = 16

        // This brings the number in the (-17, 17) range
        let num = num % PRIME as i32;

        // This brings the number in the [0, 17*2) range
        let num = num + PRIME as i32;

        Self::from(num as u8)
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
        if self == Self::zero() || rhs == Self::zero() {
            return Self::zero();
        }

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

/// Describes a cyclic multiplicative subgroup of the multiplicative group in
/// BaseField (i.e. {1, ..., 16}).
pub struct CyclicGroup {
    pub elements: Vec<BaseField>,
}

impl CyclicGroup {
    pub fn new(size: u8) -> anyhow::Result<Self> {
        // In our use case, 4 will be the original domain size, and 8 will be the extended domain (with LDE)
        if size != 4 && size != 8 {
            bail!("Unsupported group size: {size}")
        }

        if size == 4 {
            // generator: 13
            Ok(Self {
                elements: vec![1.into(), 13.into(), 16.into(), 4.into()],
            })
        } else
        /* if size == 8 */
        {
            // Notice: 1, 4 and 13 are also found in the original domain. If we
            // use this domain, we will leak the data at those point (since the
            // polynomial will evaluate to the original datum). Therefore, we
            // will want to use a coset of this subgroup. Turns out that by
            // shifting the group by 3, we get a different set.
            //
            // Remember: cosets (i.e. "a shifted group") are either equal or
            // disjoint from the original group
            //
            // Generator: 9
            let group = Self {
                elements: vec![
                    1.into(),
                    9.into(),
                    13.into(),
                    15.into(),
                    16.into(),
                    8.into(),
                    4.into(),
                    2.into(),
                ],
            };

            Ok(group.shift(3.into()))
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Shifts the group by `element`. In other words, this gives the cosets of
    /// our cyclic group (under the assumption that our cyclic group is a
    /// subgroup of {1, ... , 16})
    pub fn shift(self, g: BaseField) -> Self {
        let shifted_elements = self
            .elements
            .into_iter()
            .map(|element| element * g)
            .collect();

        Self {
            elements: shifted_elements,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_i32() {
        let ele = BaseField::from(-1);
        assert_eq!(ele, BaseField::from(16u8));

        let ele = BaseField::from(-100);
        assert_eq!(ele, BaseField::from(2u8));
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
    fn test_mult_inv() {
        for i in 1..PRIME {
            let fel = BaseField::from(i);

            assert_eq!(BaseField::one(), fel * fel.mult_inv());
        }
    }

    #[test]
    fn test_additive_inv() {
        for i in 0..PRIME {
            let fel = BaseField::from(i);

            assert_eq!(BaseField::zero(), fel + fel.minus());
        }
    }

    #[test]
    fn test_group_shift() {
        assert_eq!(
            CyclicGroup::new(8).unwrap().elements,
            vec![
                3.into(),
                10.into(),
                5.into(),
                11.into(),
                14.into(),
                7.into(),
                12.into(),
                6.into(),
            ]
        );
    }
}
