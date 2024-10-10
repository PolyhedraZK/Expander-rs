use arith::Field;
use ark_std::{log2, rand::RngCore};

use crate::EqPolynomial;

#[derive(Debug, Clone)]
pub struct MultiLinearPoly<F: Field> {
    pub coeffs: Vec<F>,
}

impl<F: Field> MultiLinearPoly<F> {
    /// Sample a random polynomials.

    #[inline]
    pub fn random(nv: usize, mut rng: impl RngCore) -> Self {
        let coeff = (0..1 << nv).map(|_| F::random_unsafe(&mut rng)).collect();
        Self { coeffs: coeff }
    }

    #[inline]
    pub fn get_num_vars(&self) -> usize {
        log2(self.coeffs.len()) as usize
    }

    // TODO: optimize this function
    #[inline]
    pub fn interpolate_over_hypercube_impl(evals: &[F]) -> Vec<F> {
        let mut coeffs = evals.to_vec();
        let num_vars = log2(evals.len());

        for i in 1..=num_vars {
            let chunk_size = 1 << i;

            coeffs.chunks_mut(chunk_size).for_each(|chunk| {
                let half_chunk = chunk_size >> 1;
                let (left, right) = chunk.split_at_mut(half_chunk);
                right
                    .iter_mut()
                    .zip(left.iter())
                    .for_each(|(a, b)| *a -= *b);
            })
        }

        coeffs
    }

    // interpolate Z evaluations over boolean hypercube {0, 1}^n
    #[inline]
    pub fn interpolate_over_hypercube(&self) -> Vec<F> {
        // Take eq poly as an example:
        //
        // The evaluation format of an eq poly over {0, 1}^2 follows:
        // eq(\vec{r}, \vec{x}) with \vec{x} order in x0 x1
        //
        //     00             01            10          11
        // (1-r0)(1-r1)    (1-r0)r1      r0(1-r1)      r0r1
        //
        // The interpolated version over x0 x1 (ordered in x0 x1) follows:
        //
        //     00               01                  10                11
        // (1-r0)(1-r1)    (1-r0)(2r1-1)      (2r0-1)(1-r1)     (2r0-1)(2r1-1)

        // NOTE(Hang): I think the original implementation of this dense multilinear
        // polynomial requires a resizing of coeffs by num vars,
        // e.g., when sumchecking - the num_var reduces, while Z evals can reuse the
        // whole space, which means we cannot simply relying Z's size itself.
        Self::interpolate_over_hypercube_impl(&self.coeffs)
    }

    /// Evaluate the polynomial at the top variable
    #[inline]
    pub fn fix_top_variable(&mut self, r: &F) {
        let n = self.coeffs.len() / 2;
        let (left, right) = self.coeffs.split_at_mut(n);

        left.iter_mut().zip(right.iter()).for_each(|(a, b)| {
            *a += *r * (*b - *a);
        });
        self.coeffs.truncate(n);
    }

    /// Hyperplonk's implementation
    /// Evaluate the polynomial at a set of variables, from bottom to top
    /// This is equivalent to `evaluate` when partial_point.len() = nv
    #[inline]
    pub fn fix_variables(&mut self, partial_point: &[F]) {
        // evaluate single variable of partial point from left to right
        partial_point
            .iter()
            .rev() // need to reverse the order of the point
            .for_each(|point| self.fix_top_variable(point));
    }

    /// Jolt's implementation
    /// Evaluate the polynomial at a set of variables, from bottom to top
    /// This is equivalent to `evaluate_with_buffer`, but slower
    /// returns Z(r) in O(n) time
    #[inline]
    pub fn evaluate_jolt(&self, r: &[F]) -> F {
        // r must have a value for each variable
        assert_eq!(r.len(), self.get_num_vars());
        let chis = EqPolynomial::evals_jolt(r);
        assert_eq!(chis.len(), self.coeffs.len());
        self.coeffs
            .iter()
            .zip(chis.iter())
            .map(|(c, chi)| *c * *chi)
            .sum()
    }

    #[inline]
    /// Expander's implementation
    /// Generic method to evaluate a multilinear polynomial.
    /// This is the preferred method to evaluate a multilinear polynomial as it does not require additional memory.
    pub fn evaluate_with_buffer(evals: &[F], point: &[F], scratch: &mut [F]) -> F {
        assert_eq!(1 << point.len(), evals.len());
        assert_eq!(evals.len(), scratch.len());

        if point.is_empty() {
            evals[0]
        } else {
            for i in 0..(evals.len() >> 1) {
                scratch[i] = (evals[i * 2 + 1] - evals[i * 2]) * point[0] + evals[i * 2];
            }

            let mut cur_eval_size = evals.len() >> 2;
            for r in point.iter().skip(1) {
                for i in 0..cur_eval_size {
                    scratch[i] = scratch[i * 2] + (scratch[i * 2 + 1] - scratch[i * 2]) * r;
                }
                cur_eval_size >>= 1;
            }
            scratch[0]
        }
    }
}
