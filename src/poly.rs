use std::{
    cmp::min,
    ops::{Add, Mul},
};

use anyhow::bail;

use crate::field::{BaseField, CyclicGroup};

#[derive(Clone, Debug)]
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

    pub fn lagrange_interp(
        domain: CyclicGroup,
        evaluations: Vec<BaseField>,
    ) -> anyhow::Result<Self> {
        if domain.len() != evaluations.len() {
            bail!("domain and evaluations have different sizes");
        }

        todo!()
    }

    pub fn degree(&self) -> usize {
        self.coefficients.len() - 1
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
}
