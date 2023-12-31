use std::{
    cmp::min,
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, MulAssign},
};

use anyhow::bail;

use crate::field::BaseField;

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

    /// Evaluates the polynomial at `x`
    pub fn eval(&self, x: BaseField) -> BaseField {
        let mut result = BaseField::zero();

        for (i, coeff) in self.coefficients.iter().enumerate() {
            result += *coeff * x.exp(i as u8)
        }

        result
    }

    /// Convenience function that evaluates the polynomial over a domain
    pub fn eval_domain(&self, domain: &[BaseField]) -> Vec<BaseField> {
        domain
            .iter()
            .map(|domain_ele| self.eval(*domain_ele))
            .collect()
    }

    // https://mathworld.wolfram.com/LagrangeInterpolatingPolynomial.html
    pub fn lagrange_interp(
        domain: &[BaseField],
        evaluations: &[BaseField],
    ) -> anyhow::Result<Self> {
        if domain.len() != evaluations.len() {
            bail!("domain and evaluations have different sizes");
        }

        let interpolated_poly = (0..domain.len())
            .map(|j| Self::partial_lagrange_poly(j, domain, evaluations))
            .sum();

        Ok(interpolated_poly)
    }

    fn partial_lagrange_poly(j: usize, domain: &[BaseField], evaluations: &[BaseField]) -> Self {
        let x_j = domain[j];
        let y_j = evaluations[j];

        let (numerator, denominator) = {
            let mut numerator = Polynomial::one();
            let mut denominator = BaseField::one();

            for domain_ele in domain.iter() {
                if x_j != *domain_ele {
                    // x - x_k
                    numerator *= Polynomial::new(vec![domain_ele.minus(), 1.into()]);

                    denominator *= x_j - *domain_ele;
                }
            }

            (numerator, denominator)
        };

        (numerator * y_j) / denominator
    }

    /// Performs one FRI step on the polynomial.
    ///
    /// For example, given initial polynomial
    ///   p(x) = 5x^3 + 4x^2 + 3x + 7
    ///
    /// We generate the two polynomials:
    ///   even_poly(x) = 4x + 7
    ///   odd_poly(x) = 5x + 3
    ///
    /// And the output polynomial is:
    ///   output_poly(x) = (4 + 5*beta)x + (7 + 3*beta)
    ///
    /// Precondition: The polynomial is not a constant (i.e. only one coefficient).
    pub fn fri_step(self, beta: BaseField) -> Self {
        assert!(
            self.coefficients.len() > 1,
            "num coefficients: {}",
            self.coefficients.len()
        );

        println!(
            "FRI step on coefficients {:?} with beta={beta}",
            self.coefficients
        );

        let even_coeffs: Vec<_> = self.coefficients.clone().into_iter().step_by(2).collect();
        let odd_coeffs: Vec<_> = self.coefficients.into_iter().skip(1).step_by(2).collect();

        let even_poly = Polynomial::new(even_coeffs);
        let odd_poly = Polynomial::new(odd_coeffs);

        even_poly + (odd_poly * beta)
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

impl Mul<BaseField> for Polynomial {
    type Output = Self;

    fn mul(self, rhs: BaseField) -> Self::Output {
        // To multiply by a scalar, we create a degree-0 polynomial, and use
        // polynomial multiplication
        let scalar_mul_poly = Self::new(vec![rhs]);

        self * scalar_mul_poly
    }
}

impl MulAssign<BaseField> for Polynomial {
    fn mul_assign(&mut self, rhs: BaseField) {
        *self = self.clone() * rhs;
    }
}

impl Div<BaseField> for Polynomial {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: BaseField) -> Self::Output {
        self * rhs.mult_inv()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::DOMAIN_TRACE;

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
    pub fn poly_mul() {
        // x - 13
        let poly_1 = Polynomial::new(vec![(-13).into(), 1.into()]);
        // x - 16
        let poly_2 = Polynomial::new(vec![(-16).into(), 1.into()]);

        let expected_mul_poly12 = Polynomial::new(vec![4.into(), 5.into(), 1.into()]);

        assert_eq!(expected_mul_poly12, poly_1.clone() * poly_2.clone());

        // x - 4
        let poly_3 = Polynomial::new(vec![(-4).into(), 1.into()]);

        let expected_mul_poly123 = Polynomial::new(vec![1.into(), 1.into(), 1.into(), 1.into()]);

        assert_eq!(expected_mul_poly123, expected_mul_poly12 * poly_3.clone());

        // Ensure associativity
        assert_eq!(
            (poly_1.clone() * poly_2.clone()) * poly_3.clone(),
            poly_1 * (poly_2 * poly_3)
        );
    }

    /// Same as poly_mul(), except uses *= operator
    #[test]
    pub fn poly_mul_assign() {
        let mut result = Polynomial::one();

        // x - 13
        let poly_1 = Polynomial::new(vec![(-13).into(), 1.into()]);
        // x - 16
        let poly_2 = Polynomial::new(vec![(-16).into(), 1.into()]);

        // x - 4
        let poly_3 = Polynomial::new(vec![(-4).into(), 1.into()]);

        result *= poly_1;
        result *= poly_2;
        result *= poly_3;

        let expected_mul_poly123 = Polynomial::new(vec![1.into(), 1.into(), 1.into(), 1.into()]);

        assert_eq!(expected_mul_poly123, result);
    }

    // Ensures that Poly::one() * any_polynomial = any_polynomial
    #[test]
    pub fn poly_mul_by_one() {
        // x - 13
        let poly_1 = Polynomial::new(vec![(-13).into(), 1.into()]);
        // x - 16
        let poly_2 = Polynomial::new(vec![1.into(), 2.into(), 3.into()]);

        assert_eq!(poly_1.clone(), Polynomial::one() * poly_1);
        assert_eq!(poly_2.clone(), Polynomial::one() * poly_2);
    }

    #[test]
    pub fn lagrange_interp() {
        let evaluations: Vec<BaseField> = vec![3.into(), 9.into(), 13.into(), 16.into()];

        let interp_poly = Polynomial::lagrange_interp(&DOMAIN_TRACE, &evaluations).unwrap();

        assert_eq!(interp_poly.eval(DOMAIN_TRACE[0]), evaluations[0]);

        assert_eq!(
            interp_poly,
            Polynomial::new(vec![6.into(), 16.into(), 2.into(), 13.into()])
        );
    }

    #[test]
    pub fn fri_step_deg_3() {
        let poly = Polynomial::new(vec![1.into(), 2.into(), 3.into(), 4.into()]);
        let beta = BaseField::from(7u8);

        let expected_poly = Polynomial::new(vec![
            BaseField::from(1) + BaseField::from(2) * beta,
            BaseField::from(3) + BaseField::from(4) * beta,
        ]);

        assert_eq!(expected_poly, poly.fri_step(beta));
    }

    #[test]
    pub fn fri_step_deg_1() {
        let poly = Polynomial::new(vec![2.into(), 3.into()]);
        let beta = BaseField::from(7u8);

        let expected_poly = Polynomial::new(vec![BaseField::from(2) + BaseField::from(3) * beta]);

        assert_eq!(expected_poly, poly.fri_step(beta));
    }
}
