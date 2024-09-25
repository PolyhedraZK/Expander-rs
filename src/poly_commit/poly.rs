use ark_std::{end_timer, start_timer};

use crate::GKRConfig;
use arith::{Field, SimdField};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MultiLinearPoly {}

impl MultiLinearPoly {
    pub fn eval_generic<F: Field>(evals: &[F], x: &[F], scratch: &mut [F]) -> F {
        assert_eq!(1 << x.len(), evals.len());
        assert_eq!(evals.len(), scratch.len());

        if x.is_empty() {
            evals[0]
        } else {
            for i in 0..(evals.len() >> 1) {
                scratch[i] = (evals[i * 2 + 1] - evals[i * 2]) * x[0] + evals[i * 2];
            }

            let mut cur_eval_size = evals.len() >> 2;
            for r in x.iter().skip(1) {
                for i in 0..cur_eval_size {
                    scratch[i] = scratch[i * 2] + (scratch[i * 2 + 1] - scratch[i * 2]) * r;
                }
                cur_eval_size >>= 1;
            }
            scratch[0]
        }
    }

    pub fn eval_circuit_vals_at_challenge<C: GKRConfig>(
        evals: &[C::SimdCircuitField],
        x: &[C::ChallengeField],
        scratch: &mut [C::Field],
    ) -> C::Field {
        let timer = start_timer!(|| format!("eval mle with {} vars", x.len()));
        assert_eq!(1 << x.len(), evals.len());

        let ret = if x.is_empty() {
            C::simd_circuit_field_into_field(&evals[0])
        } else {
            for i in 0..(evals.len() >> 1) {
                scratch[i] = C::field_add_simd_circuit_field(
                    &C::simd_circuit_field_mul_challenge_field(
                        &(evals[i * 2 + 1] - evals[i * 2]),
                        &x[0],
                    ),
                    &evals[i * 2],
                );
            }

            let mut cur_eval_size = evals.len() >> 2;
            for r in x.iter().skip(1) {
                for i in 0..cur_eval_size {
                    scratch[i] = scratch[i * 2] + (scratch[i * 2 + 1] - scratch[i * 2]).scale(r);
                }
                cur_eval_size >>= 1;
            }
            scratch[0]
        };
        end_timer!(timer);

        ret
    }
}
