use std::{
    cmp::min,
    iter::Sum,
    ops::{Add, AddAssign, Mul, MulAssign},
};

use anyhow::bail;

use crate::field::{BaseField, CyclicGroup};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial {
    // for
    // p(x) = a + bx + cx^2
    // coefficients: [a, b, c]
    coefficients: Vec<BaseField>,
}

impl Polynomial {
    pub fn new(coefficients: Vec<BaseField>) -> Self {
        Self { coefficients }
    }

    pub fn zero() -> Self {
        Self {
            coefficients: vec![0.into()],
        }
    }

    pub fn one() -> Self {
        Self {
            coefficients: vec![1.into()],
        }
    }

    pub fn degree(&self) -> usize {
        self.coefficients.len() - 1
    }

    pub fn scalar_mul(&mut self, x: BaseField) {
        let scalar_mul_poly = Self::new(vec![x]);

        *self *= scalar_mul_poly;
    }

    pub fn scalar_div(&mut self, x: BaseField) {
        self.scalar_mul(x.mult_inv())
    }

    /// Evaluates the polynomial at `x`
    pub fn eval(&self, x: BaseField) -> BaseField {
        let mut result = BaseField::zero();

        for (i, coeff) in self.coefficients.iter().enumerate() {
            result += *coeff * x.exp(i as u8)
        }

        result
    }

    // https://mathworld.wolfram.com/LagrangeInterpolatingPolynomial.html
    pub fn lagrange_interp(
        domain: &CyclicGroup,
        evaluations: &[BaseField],
    ) -> anyhow::Result<Self> {
        if domain.len() != evaluations.len() {
            bail!("domain and evaluations have different sizes");
        }

        let interpolated_poly = (0..domain.len())
            .into_iter()
            .map(|j| Self::sub_lagrange_poly(j, &domain, evaluations))
            .sum();

        Ok(interpolated_poly)
    }

    fn sub_lagrange_poly(j: usize, domain: &CyclicGroup, evaluations: &[BaseField]) -> Self {
        let x_j = domain.elements[j];
        let y_j = evaluations[j];

        let (numerator, denominator) = {
            let mut numerator = Polynomial::one();
            let mut denominator = BaseField::one();

            for domain_ele in domain.elements.iter() {
                // x - x_k
                numerator *= Polynomial::new(vec![BaseField::from(-1) * *domain_ele, 1.into()]);

                if x_j != *domain_ele {
                    denominator *= x_j - *domain_ele;
                }
            }

            (numerator, denominator)
        };

        let mut out_poly = numerator;
        out_poly.scalar_mul(y_j);
        out_poly.scalar_div(denominator);

        out_poly
    }
}

impl Add for Polynomial {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut coefficients_sum = Vec::new();

        let min_coeffs_len = min(self.coefficients.len(), rhs.coefficients.len());

        for i in 0..min_coeffs_len {
            coefficients_sum.push(self.coefficients[i] + rhs.coefficients[i]);
        }

        if self.coefficients.len() > min_coeffs_len {
            coefficients_sum.extend_from_slice(&self.coefficients[min_coeffs_len..])
        }

        if rhs.coefficients.len() > min_coeffs_len {
            coefficients_sum.extend_from_slice(&rhs.coefficients[min_coeffs_len..])
        }

        Self {
            coefficients: coefficients_sum,
        }
    }
}

impl AddAssign for Polynomial {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl MulAssign for Polynomial {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}

impl Sum for Polynomial {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = Self::zero();

        for poly in iter {
            total += poly;
        }

        total
    }
}

impl Mul for Polynomial {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mul_degree = self.degree() + rhs.degree();

        let mut mul_coeffs: Vec<BaseField> = vec![0.into(); mul_degree + 1];

        for (idx_lhs, coeff_lhs) in self.coefficients.iter().enumerate() {
            for (idx_rhs, coeff_rhs) in rhs.coefficients.iter().enumerate() {
                // e.g. (ax^2) * (bx^3) = ab x^5
                mul_coeffs[idx_lhs + idx_rhs] += *coeff_lhs * *coeff_rhs;
            }
        }

        Self {
            coefficients: mul_coeffs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn poly_add_self() {
        let poly_1 = Polynomial::new(vec![1.into(), 2.into(), 3.into()]);
        let poly_2 = poly_1.clone();

        let sum_poly = poly_1 + poly_2;

        assert_eq!(sum_poly.coefficients, vec![2.into(), 4.into(), 6.into()])
    }

    #[test]
    pub fn poly_add_diff_degree() {
        let poly_1 = Polynomial::new(vec![1.into(), 2.into(), 3.into()]);
        let poly_2 = Polynomial::new(vec![
            0.into(),
            0.into(),
            0.into(),
            4.into(),
            5.into(),
            6.into(),
        ]);

        let sum_poly = poly_1 + poly_2;

        assert_eq!(
            sum_poly.coefficients,
            vec![1.into(), 2.into(), 3.into(), 4.into(), 5.into(), 6.into()]
        )
    }

    #[test]
    pub fn poly_mul_self() {
        let poly_1 = Polynomial::new(vec![1.into(), 2.into(), 3.into()]);
        let poly_2 = poly_1.clone();

        let mul_poly = poly_1 * poly_2;

        assert_eq!(
            mul_poly.coefficients,
            vec![1.into(), 4.into(), 10.into(), 12.into(), 9.into()]
        )
    }

    #[test]
    pub fn test_lagrange_interp() {
        let domain = CyclicGroup::new(4).unwrap();
        let evaluations: Vec<BaseField> = vec![3.into(), 9.into(), 13.into(), 16.into()];

        let interp_poly = Polynomial::lagrange_interp(&domain, &evaluations).unwrap();

        assert_eq!(interp_poly.eval(domain.elements[0]), evaluations[0]);

        assert_eq!(
            interp_poly,
            Polynomial::new(vec![6.into(), 16.into(), 2.into(), 13.into()])
        );
    }

    #[test]
    pub fn dummy_temp() {}
}
